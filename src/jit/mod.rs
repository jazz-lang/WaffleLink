macro_rules! op_unreachable {
    () => {
        unsafe { std::hint::unreachable_unchecked() };
    };
}
use crate::value::Value;
use std::collections::HashMap;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod jit_x86;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use jit_x86::*;
pub mod add_generator;
pub mod arithmetic;
pub mod call;
#[cfg(target_pointer_width = "64")]
pub mod jit64;
pub mod mathic;
pub mod mul_generator;
pub mod operations;
pub mod sub_generator;
#[cfg(target_pointer_width = "64")]
pub mod tail_call64;
use crate::builtins::WResult;
use crate::bytecode::*;
use crate::interpreter::callframe::*;
use crate::interpreter::stack_alignment::*;
pub type JITFunction = extern "C" fn(&mut CallFrame) -> WResult;
pub type JITTrampoline = extern "C" fn(&mut CallFrame, usize) -> WResult;

pub extern "C" fn safepoint_slow_path(_sp: *mut u8) {}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum JITType {
    Interp,
    Baseline,
    DFG,
}

pub fn jit_frame_register_count_for(c: &CodeBlock) -> usize {
    return round_local_reg_count_for_frame_pointer_offset(
        c.callee_locals as usize + MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL_IN_REGISTERS,
    );
}

#[cfg(all(target_arch = "x86_64", windows))]
pub const MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL: usize = 64; // 4 args in registers, but stack space needs to be allocated for all args.

#[cfg(all(target_arch = "x86_64", not(windows)))]
pub const MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL: usize = 0; // all args in registers
#[cfg(target_arch = "x86")]
pub const MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL: usize = 40; // 7 args on stack (28 bytes)
#[cfg(target_arch = "aarch64")]
pub const MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL: usize = 0; // all args in registers
#[cfg(target_arch = "arm")]
pub const MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL: usize = 24; // First four args in registers, remaining 4 args on stack.
#[cfg(target_arch = "mips")]
pub const MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL: usize = 40; // Though args are in registers, there need to be space on the stack for all args.

pub const MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL_IN_REGISTERS: usize =
    MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL / 8;

pub struct CallRecord {
    pub from: masm::Call,
    pub idx: usize,
    pub callee: usize,
}

pub struct JumpTable {
    pub from: Jump,
    pub to_bytecode_offset: u32,
}

#[derive(Copy, Clone)]
pub struct SlowCaseEntry {
    pub from: Jump,
    pub to: u32,
}

#[derive(Default)]
pub struct CallCompilationInfo {
    pub hot_path_begin: DataLabelPtr,
    pub hot_path_other: masm::Call,
    pub call_return_location: masm::Call,
}

impl<'a> JIT<'a> {
    pub fn branch_ptr_with_patch(
        &mut self,
        cond: RelationalCondition,
        left: Reg,
        data_label: &mut DataLabelPtr,
        initial: usize,
    ) -> Jump {
        *data_label = self.masm.move_with_patch_ptr(initial, SCRATCH_REG);
        return self.masm.branch64(cond, left, SCRATCH_REG);
    }
    pub fn address_for_vreg(src: virtual_register::VirtualRegister) -> Mem {
        return Mem::Base(BP, src.offset() * std::mem::size_of::<u64>() as i32);
    }
    pub fn emit_get_virtual_register(&mut self, src: virtual_register::VirtualRegister, dest: Reg) {
        if src.is_constant() {
            let value = self.code_block.get_constant(src);
            self.masm.move_i64(unsafe { value.u.as_int64 }, dest);
        } else {
            self.masm.load64(Self::address_for_vreg(src), dest);
        }
    }

    pub fn emit_put_virtual_register(&mut self, dst: virtual_register::VirtualRegister, from: Reg) {
        self.masm.store64(from, Self::address_for_vreg(dst));
    }

    pub fn emit_function_prologue(&mut self) {
        self.masm.push(BP);
        self.masm.move_rr(SP, BP);
    }
    pub fn stack_pointer_offset_for(&self, code_block: &CodeBlock) -> i32 {
        virtual_register::virtual_register_for_local(
            jit_frame_register_count_for(code_block) as i32 - 1,
        )
        .offset()
    }

    pub fn materialize_tag_check_regs(&mut self) {
        #[cfg(feature = "value64")]
        {
            self.masm.move_i64(Value::NUMBER_TAG, NUMBER_TAG_REGISTER);
            self.masm.or64_imm32(
                Value::OTHER_TAG as _,
                NUMBER_TAG_REGISTER,
                NOT_CELL_MASK_REGISTER,
            );
        }
    }
    pub fn compile_without_linking(&mut self) {
        self.emit_function_prologue();
        self.masm.move_i64(self.code_block as *const _ as i64, T0);
        self.masm.store64(
            T0,
            Mem::Base(
                BP,
                -virtual_register::virtual_register_for_local(1).offset() * 8,
            ),
        );
        self.labels = Vec::with_capacity(self.code_block.instructions.len());
        self.labels
            .resize(self.code_block.instructions.len(), Label::default());
        let frame_top_offset = self.stack_pointer_offset_for(self.code_block) * 8;
        // let max_frame_size = -frame_top_offset;
        #[cfg(target_pointer_width = "64")]
        {
            self.masm.add64_imm32(frame_top_offset, BP, T1);
        }
        #[cfg(target_pointer_width = "32")]
        {
            self.masm.add32i(frame_top_offset, BP, T1);
        }
        let mut stack_overflow = JumpList::new();
        // TODO: Check for stack overflow
        self.masm.move_rr(T1, SP);
        self.materialize_tag_check_regs();

        self.private_compile_bytecode();
        self.private_compile_link_pass();
        self.private_compile_slow_cases();
        stack_overflow.link(&mut self.masm);
        if MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL != 0 {
            #[cfg(target_pointer_width = "64")]
            {
                self.masm
                    .add64_imm32(-(MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL as i32), SP, SP);
            }
            #[cfg(target_pointer_width = "32")]
            {
                self.masm
                    .add32i(-(MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL as i32), SP, SP);
            }
        }

        self.link_buffer = JITLinkBuffer::from_masm(&mut self.masm);
    }
    pub fn update_top_frame(&mut self) {
        self.masm.move_i64(
            &crate::get_vm().top_call_frame as *const *mut _ as i64,
            SCRATCH_REG,
        );
        self.masm.store64(BP, Mem::Base(SCRATCH_REG, 0));
    }
    fn private_compile_link_pass(&mut self) {
        for i in 0..self.jmptable.len() {
            self.jmptable[i].from.link_to(
                &mut self.masm,
                self.labels[self.jmptable[i].to_bytecode_offset as usize],
            );
        }
        self.jmptable.clear();
    }
    fn private_compile_bytecode(&mut self) {
        for i in 0..self.code_block.instructions.len() {
            self.bytecode_index = i as _;
            self.labels[i] = self.masm.label();
            let ins = &self.code_block.instructions[i];
            match ins {
                Ins::Sub { .. } => self.emit_op_sub(ins),
                Ins::Add { .. } => self.emit_op_add(ins),
                Ins::Mul { .. } => self.emit_op_mul(ins),
                Ins::Return(val) => {
                    self.emit_get_virtual_register(*val, RET0);
                    self.masm.function_epilogue();
                    self.masm.ret();
                }
                Ins::Enter => {
                    let count = self.code_block.num_vars;
                    for x in 0..count {
                        self.masm.store64_imm64(
                            unsafe { Value::undefined().u.as_int64 },
                            Self::address_for_vreg(virtual_register::virtual_register_for_local(
                                x as _,
                            )),
                        );
                    }
                }
                _ => todo!(),
            }
        }
    }
    fn private_compile_slow_cases(&mut self) {
        // SAFE: we do not mutate slow_cases when generating slow paths.
        let slow_cases = unsafe { &*(&self.slow_cases as *const Vec<SlowCaseEntry>) };
        let mut iter = slow_cases.iter().peekable();
        while let Some(case) = iter.peek() {
            self.bytecode_index = case.to as _;
            let curr = &self.code_block.instructions[self.bytecode_index];
            match curr {
                Ins::Add(_src1, _src2, _dest) => {
                    self.emit_slow_op_add(curr, &mut iter);
                    self.bytecode_index += 1;
                }
                Ins::Sub { .. } => {
                    self.emit_slow_op_sub(curr, &mut iter);
                    self.bytecode_index += 1;
                }
                Ins::Mul { .. } => {
                    self.emit_slow_op_mul(curr, &mut iter);
                    self.bytecode_index += 1;
                }
                _ => (),
            }
            let jump = self.masm.jump();
            self.emit_jump_slow_to_hot(jump, 0);
        }
    }
    pub fn emit_jump_slow_to_hot(&mut self, j: Jump, relative_offset: i32) {
        let label = self.labels[(self.bytecode_index as i32 as i32 + relative_offset) as usize];
        j.link_to(&mut self.masm, label);
    }
    pub fn link_slow_case(&mut self, case: SlowCaseEntry) {
        if case.from.label().asm_label().is_set() {
            case.from.link(&mut self.masm);
        } else {
        }
    }

    pub fn emit_naked_call(&mut self, ptr: *const u8) -> masm::Call {
        let call = self.masm.near_call();
        self.calls.push(CallRecord {
            from: call,
            idx: self.bytecode_index,
            callee: ptr as usize,
        });
        call
    }

    pub fn link_all_slow_cases_for_bytecode_index(
        &mut self,
        iter: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
        idx: u32,
    ) {
        while let Some(item) = iter.next() {
            if item.to == idx {
                self.link_slow_case(*item);
            } else {
                break;
            }
        }
    }

    pub fn link_all_slow_cases(
        &mut self,
        iter: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
    ) {
        self.link_all_slow_cases_for_bytecode_index(iter, self.bytecode_index as _);
    }
    pub fn link(&mut self) {
        let patch_buffer = &mut self.link_buffer;
        if patch_buffer.did_fail_to_allocate() {
            panic!("Cannot allocate link buf");
        }
        while let Some(record) = self.calls.pop() {
            if record.callee != 0 {
                patch_buffer.link_call(record.from, record.callee as *const _);
            }
        }
        let mut code_map = HashMap::new();
        for off in 0..self.labels.len() {
            if self.labels[off].asm_label().is_set() {
                code_map.insert(
                    off as u32,
                    patch_buffer.location_of_label(self.labels[off].asm_label()),
                );
            }
        }
        self.link_buffer.perform_finalization();
    }

    pub fn disasm(&mut self) {
        let code = self.link_buffer.code;
        let size = self.link_buffer.size;
        let code_slice = unsafe { std::slice::from_raw_parts(code, size) };
        use capstone::prelude::*;

        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Att)
            .detail(true)
            .build()
            .expect("Failed to create Capstone object");
        let asm = cs.disasm_all(code_slice, code as _).unwrap();
        for i in asm.iter() {
            println!("{}", i);
        }
    }

    pub fn add_slow_case(&mut self, j: Jump) {
        self.slow_cases.push(SlowCaseEntry {
            from: j,
            to: self.bytecode_index as _,
        });
    }
}

pub fn disasm_code(code: *const u8, len: usize) {
    let code_slice = unsafe { std::slice::from_raw_parts(code, len) };
    use capstone::prelude::*;

    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Att)
        .detail(true)
        .build()
        .expect("Failed to create Capstone object");
    let asm = cs.disasm_all(code_slice, code as _).unwrap();
    for i in asm.iter() {
        println!("{}", i);
    }
}
