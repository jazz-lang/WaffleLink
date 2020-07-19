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
use crate::object::*;
use crate::vtable::VTable;
pub mod div_generator;
#[cfg(target_pointer_width = "64")]
pub mod jit64;
pub mod mathic;
pub mod mul_generator;
pub mod operations;
pub mod sub_generator;
#[cfg(target_pointer_width = "64")]
pub mod tail_call64;
pub mod thunk_generator;
use crate::bytecode::*;
use crate::interpreter::callframe::*;
use crate::*;
pub type JITFunction = extern "C" fn(&mut CallFrame) -> WaffleResult;
pub type JITTrampoline = extern "C" fn(&mut CallFrame, usize) -> WaffleResult;

pub extern "C" fn safepoint_slow_path(_sp: *mut u8) {}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum JITType {
    Interp,
    Baseline,
    DFG,
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

    pub fn emit_get_virtual_registers(
        &mut self,
        src1: virtual_register::VirtualRegister,
        src2: virtual_register::VirtualRegister,
        dest1: Reg,
        dest2: Reg,
    ) {
        self.emit_get_virtual_register(src1, dest1);
        self.emit_get_virtual_register(src2, dest2);
    }
    pub fn emit_get_virtual_register(&mut self, src: virtual_register::VirtualRegister, dest: Reg) {
        if src.is_constant() {
            let value = self.code_block.get_constant(src);
            self.masm.move_i64(unsafe { value.u.as_int64 }, dest);
        } else {
            if src.is_local() {
                self.masm.load64(
                    Mem::Base(REG_CALLFRAME, offset_of!(CallFrame, regs) as i32),
                    dest,
                );
                self.masm.load64(Mem::Base(dest, src.to_local() * 8), dest);
            } else {
                self.masm.load64(
                    Mem::Base(REG_CALLFRAME, offset_of!(CallFrame, args) as i32),
                    dest,
                );
                self.masm
                    .load64(Mem::Base(dest, src.to_argument() * 8), dest);
            }
        }
    }

    pub fn emit_put_virtual_register(
        &mut self,
        dst: virtual_register::VirtualRegister,
        from: Reg,
        scratch: Reg,
    ) {
        if dst.is_local() {
            self.masm.load64(
                Mem::Base(REG_CALLFRAME, offset_of!(CallFrame, regs) as i32),
                scratch,
            );
            self.masm
                .store64(from, Mem::Base(scratch, dst.to_local() * 8));
        } else {
            self.masm.load64(
                Mem::Base(REG_CALLFRAME, offset_of!(CallFrame, args) as i32),
                scratch,
            );
            self.masm
                .store64(from, Mem::Base(scratch, dst.to_argument() * 8));
        }
    }

    pub fn emit_function_prologue(&mut self) {
        self.function_prologue(0);
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
        self.masm.move_rr(AGPR0, REG_CALLFRAME);

        self.labels = Vec::with_capacity(self.code_block.instructions.len());
        self.labels
            .resize(self.code_block.instructions.len(), Label::default());

        self.materialize_tag_check_regs();

        self.private_compile_bytecode();
        self.private_compile_link_pass();
        self.private_compile_slow_cases();

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
        self.masm.store64(REG_CALLFRAME, Mem::Base(SCRATCH_REG, 0));
    }
    pub fn add_jump(&mut self, jump: Jump, relative_offset: i32) {
        self.jmptable.push(JumpTable {
            from: jump,
            to_bytecode_offset: (self.bytecode_index as i32 + relative_offset) as _,
        })
    }
    /// This function assumes RET0 is `WaffleResult.a` and `RET1` is `WaffleResult.b` or exception value.
    pub fn check_exception(&mut self, force: bool) {
        if (self.bytecode_index as u32 >= self.try_start
            && self.bytecode_index < self.try_end as usize)
            || force
        {
            assert!(crate::get_vm().exception_addr() != std::ptr::null());
            let br = self
                .masm
                .branch64_imm32(RelationalCondition::Equal, 1, RET0);
            self.exception_check.last_mut().unwrap().0.push(br);
        } else {
            let br = self
                .masm
                .branch64_imm64(RelationalCondition::NotEqual, RET0, 1);
            self.function_epilogue(AGPR0);
            if cfg!(windows) {
                self.masm
                    .store64_imm32(1, Mem::Base(AGPR0, offset_of!(crate::WaffleResult, a) as _));
                self.masm.store64(
                    RET1,
                    Mem::Base(AGPR0, offset_of!(crate::WaffleResult, b) as _),
                );
                self.masm.move_rr(AGPR0, RET0);
            }
            self.masm.ret();
            br.link(&mut self.masm);
        }
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
            self.add_comment(&format!("[{:04}] {:?}", self.bytecode_index, ins));
            match ins {
                Ins::Move(dst, src) => {
                    self.emit_get_virtual_register(*src, T0);
                    self.emit_put_virtual_register(*dst, T0, T1);
                }
                Ins::JEq(lhs, rhs, offset) => {
                    self.emit_get_virtual_registers(*lhs, *rhs, T0, T1);
                    self.emit_jump_slow_case_if_not_ints(T0, T1, T2);
                    let jump = self.masm.branch32(RelationalCondition::Equal, T0, T1);
                    self.add_jump(jump, *offset);
                }
                Ins::JNEq(lhs, rhs, offset) => {
                    self.emit_get_virtual_registers(*lhs, *rhs, T0, T1);
                    self.emit_jump_slow_case_if_not_ints(T0, T1, T2);
                    let jump = self.masm.branch32(RelationalCondition::NotEqual, T0, T1);
                    self.add_jump(jump, *offset);
                }
                Ins::Equal(lhs, rhs, dst) => {
                    self.emit_get_virtual_registers(*lhs, *rhs, T0, T1);
                    self.emit_jump_slow_case_if_not_ints(T0, T1, T2);
                    self.masm.compare32(RelationalCondition::Equal, T0, T1, T0);
                    self.box_boolean(T0, T0);
                    self.emit_put_virtual_register(*dst, T0, T1);
                }
                Ins::NotEqual(lhs, rhs, dst) => {
                    self.emit_get_virtual_registers(*lhs, *rhs, T0, T1);
                    self.emit_jump_slow_case_if_not_ints(T0, T1, T2);
                    self.masm
                        .compare32(RelationalCondition::NotEqual, T0, T1, T0);
                    self.box_boolean(T0, T0);
                    self.emit_put_virtual_register(*dst, T0, T1);
                }
                Ins::JmpIfZero(value_v, off) => {
                    let value = T0;
                    let s1 = T1;
                    let s2 = T2;
                    self.emit_get_virtual_register(*value_v, value);
                    let j = self.branch_if_falsey(value, s1, s2, FT0, FT1, false);
                    for j in j.jumps {
                        self.add_jump(j, *off);
                    }
                }
                Ins::JmpIfNotZero(value_v, off) => {
                    let value = T0;
                    let s1 = T1;
                    let s2 = T2;
                    self.emit_get_virtual_register(*value_v, value);
                    let j = self.branch_if_truthy(value, s1, s2, FT0, FT1, false);
                    for j in j.jumps {
                        self.add_jump(j, *off);
                    }
                }
                Ins::JLess { .. } => self.emit_op_jless(ins),
                Ins::JLessEq { .. } => self.emit_op_jlesseq(ins),
                Ins::JGreater { .. } => self.emit_op_jnless(ins),
                Ins::JGreaterEq { .. } => self.emit_op_jgreaterq(ins),
                Ins::Sub { .. } => self.emit_op_sub(ins),
                Ins::Add { .. } => self.emit_op_add(ins),
                Ins::Mul { .. } => self.emit_op_mul(ins),
                Ins::Div { .. } => self.emit_op_div(ins),
                Ins::Call(dest, this, callee, argc) => {
                    // TODO: Use code patching and fast path/slow path codegen for calls
                    self.masm.prepare_call_with_arg_count(3);
                    self.emit_get_virtual_register(*callee, AGPR1);
                    self.masm.pass_reg_as_arg(REG_CALLFRAME, 0);
                    self.masm.pass_int32_as_arg(*argc as _, 2);
                    self.emit_get_virtual_register(*this, AGPR3);
                    self.masm
                        .call_ptr_argc(operations::operation_call_func as *const _, 3);
                    self.check_exception(false);
                    self.emit_put_virtual_register(*dest, RET1, RET0);
                }
                Ins::Safepoint => {
                    self.masm.load64(
                        Mem::Absolute(&crate::get_vm().stop_world as *const bool as usize),
                        T0,
                    );
                    let j = self.masm.branch64_imm32(RelationalCondition::Equal, 1, T0);
                    self.add_slow_case(j);
                }
                Ins::LoopHint => {
                    #[cfg(feature = "opt-jit")]
                    {
                        if crate::get_vm().opt_jit {
                            self.masm.move_i64(self.code_block as *const _ as i64, T1);
                            self.masm.load32(
                                Mem::Base(T1, offset_of!(CodeBlock, exc_counter) as i32),
                                T0,
                            );
                            self.masm.add64_imm32(1, T0, T0);
                            self.masm.store32(
                                T0,
                                Mem::Base(T1, offset_of!(CodeBlock, exc_counter) as i32),
                            );
                            let jump =
                                self.masm
                                    .branch32_imm(RelationalCondition::Equal, 10_000_0, T0);
                            self.osr_upgrade.push(jump);
                        }
                    };
                }
                Ins::Try(off) => {
                    self.try_start = self.bytecode_index as _;
                    self.try_end = self.bytecode_index as u32 + *off;
                    self.exception_check
                        .push((Vec::with_capacity(1), (self.try_start, self.try_end)));
                }
                Ins::Catch(dest) => {
                    let (x, (prev_start, prev_end)) = self.exception_check.pop().unwrap();
                    for jump in x {
                        jump.link(&mut self.masm);
                    }
                    self.emit_put_virtual_register(*dest, RET1, RET0);
                    self.try_start = prev_start;
                    self.try_end = prev_end;
                }
                Ins::Throw(reg) => {
                    self.emit_get_virtual_register(*reg, NON_CALLEE_SAVE_T0);
                    if cfg!(not(windows)) {
                        self.masm.move_i32(1, RET0);
                        self.masm.move_rr(NON_CALLEE_SAVE_T0, RET1);
                    }
                    self.function_epilogue(AGPR0);
                    if cfg!(windows) {
                        self.masm.store64_imm32(
                            1,
                            Mem::Base(AGPR0, offset_of!(crate::WaffleResult, a) as _),
                        );
                        self.masm.store64(
                            NON_CALLEE_SAVE_T0,
                            Mem::Base(AGPR0, offset_of!(crate::WaffleResult, b) as _),
                        );
                        self.masm.move_rr(AGPR0, RET0);
                    }
                    self.masm.ret();
                }
                Ins::Return(val) => {
                    self.emit_get_virtual_register(*val, NON_CALLEE_SAVE_T0);
                    self.function_epilogue(RET0);
                    if cfg!(windows) {
                        self.masm.store64_imm32(
                            0,
                            Mem::Base(RET0, offset_of!(crate::WaffleResult, a) as _),
                        );
                        self.masm.store64(
                            NON_CALLEE_SAVE_T0,
                            Mem::Base(RET0, offset_of!(crate::WaffleResult, b) as _),
                        );
                    } else {
                        self.masm.move_rr(NON_CALLEE_SAVE_T0, RET1);
                        self.masm.move_i64(0, RET0);
                    }
                    self.masm.ret();
                }
                Ins::Enter => {}
                _ => todo!(),
            }
        }
        self.add_comment("\t(End of main path)");
    }
    fn private_compile_slow_cases(&mut self) {
        if self.slow_cases.is_empty() {
            return;
        }
        // SAFE: we do not mutate slow_cases when generating slow paths.
        let slow_cases = unsafe { &*(&self.slow_cases as *const Vec<SlowCaseEntry>) };
        let mut iter = slow_cases.iter().peekable();
        while let Some(case) = iter.peek() {
            self.bytecode_index = case.to as _;
            let curr = &self.code_block.instructions[self.bytecode_index];
            self.add_comment(&format!("(S) [{:04}] {:?}", self.bytecode_index, curr));
            match curr {
                Ins::JEq(_, _, off) => {
                    self.link_all_slow_cases(&mut iter);
                    self.masm.prepare_call_with_arg_count(2);
                    self.masm.pass_reg_as_arg(T0, 0);
                    self.masm.pass_reg_as_arg(T1, 1);
                    self.masm
                        .call_ptr_argc(operations::operation_compare_eq as _, 2);
                    let j = self
                        .masm
                        .branch32_test(ResultCondition::NonZero, RET0, RET0);
                    self.emit_jump_slow_to_hot(j, *off);
                }
                Ins::JNEq(_, _, off) => {
                    self.link_all_slow_cases(&mut iter);
                    self.masm.prepare_call_with_arg_count(2);
                    self.masm.pass_reg_as_arg(T0, 0);
                    self.masm.pass_reg_as_arg(T1, 1);
                    self.masm
                        .call_ptr_argc(operations::operation_compare_neq as _, 2);
                    let j = self
                        .masm
                        .branch32_test(ResultCondition::NonZero, RET0, RET0);
                    self.emit_jump_slow_to_hot(j, *off);
                }
                Ins::JLess { .. } => {
                    self.emit_slow_op_jless(curr, &mut iter);
                    self.bytecode_index += 1;
                }
                Ins::Add(_src1, _src2, _dest) => {
                    self.emit_slow_op_add(curr, &mut iter);
                    self.bytecode_index += 1;
                }
                Ins::Div(..) => {
                    self.link_all_slow_cases(&mut iter);
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
                Ins::Safepoint => {
                    self.link_all_slow_cases(&mut iter);
                    extern "C" fn safepoint(vm: &crate::VM, stack_top: *const u8) {}
                    self.masm.prepare_call_with_arg_count(2);
                    self.masm
                        .pass_ptr_as_arg(crate::get_vm() as *const _ as usize, 0);
                    self.masm.pass_reg_as_arg(SP, 1);
                    self.masm.call_ptr_argc(safepoint as *const _, 2);
                    self.bytecode_index += 1;
                }
                _ => (),
            }
            let jump = self.masm.jump();
            self.emit_jump_slow_to_hot(jump, 0);
        }
        self.add_comment("\t(End of Slow Path)");
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
        self.code_block.jit_data().code_map = code_map;
        self.code_block.jit_data().executable_addr = self.link_buffer.code as usize;
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
            if let Some(c) = self.get_comment_for((i.address() - code as u64) as u32) {
                println!("{}", c);
            }
            println!("\t{}", i);
        }
    }

    pub fn add_slow_case(&mut self, j: Jump) {
        self.slow_cases.push(SlowCaseEntry {
            from: j,
            to: self.bytecode_index as _,
        });
    }
    pub fn add_slow_cases(&mut self, j: &Vec<Jump>) {
        for j in j.iter() {
            self.slow_cases.push(SlowCaseEntry {
                from: *j,
                to: self.bytecode_index as _,
            });
        }
    }
    pub fn branch_if_type(&mut self, value: Reg, vtable: &VTable) -> Jump {
        self.masm.branch64_imm64_mem(
            RelationalCondition::Equal,
            vtable as *const _ as i64,
            Mem::Base(value, offset_of!(Obj, vtable) as i32),
        )
    }
    pub fn branch_if_not_type(&mut self, value: Reg, vtable: &VTable) -> Jump {
        self.masm.branch64_imm64_mem(
            RelationalCondition::NotEqual,
            vtable as *const _ as i64,
            Mem::Base(value, offset_of!(Obj, vtable) as i32),
        )
    }

    pub fn branch_if_string(&mut self, value: Reg) -> Jump {
        self.branch_if_type(value, &crate::builtins::STRING_VTBL)
    }
    pub fn branch_if_not_string(&mut self, value: Reg) -> Jump {
        self.branch_if_not_type(value, &crate::builtins::STRING_VTBL)
    }
    pub fn branch_if_heap_bigint(&mut self, value: Reg) -> Jump {
        self.branch_if_type(value, &crate::bigint::BIGINT_VTBL)
    }
    pub fn branch_if_value(
        &mut self,
        value: Reg,
        scratch: Reg,
        scratch2: Reg,
        value_as_fpr: FPReg,
        temp_fpr: FPReg,
        should_check_masquerades_as_undefined: bool,
        invert: bool,
    ) -> JumpList {
        // Implements the following control flow structure:
        // if (value is cell) {
        //     if (value is string or value is HeapBigInt)
        //         result = !value->length
        //     else {
        //         do evil things for masquerades-as-undefined
        //         result = true
        //     }
        // } else if (value is int32) {
        //     result = !unboxInt32(value)
        // } else if (value is number) {
        //     result = !unboxDouble(value)
        // } else if (value is BigInt32) {
        //     result = !unboxBigInt32(value)
        // } else {
        //     result = value == jsTrue
        // }
        let mut done = JumpList::new();
        let mut truthy = JumpList::new();
        let not_cell = self.branch_if_not_cell(value, true);
        let is_string = self.branch_if_string(value);
        if should_check_masquerades_as_undefined {
            todo!();
        } else {
            if invert {
                truthy.push(self.masm.jump());
            } else {
                done.push(self.masm.jump());
            }
        }
        is_string.link(&mut self.masm);
        let j = self.masm.branch64_imm64(
            if invert {
                RelationalCondition::Equal
            } else {
                RelationalCondition::NotEqual
            },
            value,
            unsafe { crate::get_vm().empty_string.u.as_int64 },
        );
        truthy.push(j);
        done.push(self.masm.jump());

        not_cell.link(&mut self.masm);
        let not_int32 = self.branch_if_not_int32(value, true);
        truthy.push(self.masm.branch32_test(
            if invert {
                ResultCondition::Zero
            } else {
                ResultCondition::NonZero
            },
            value,
            value,
        ));
        done.push(self.masm.jump());
        not_int32.link(&mut self.masm);
        let not_double = self.branch_if_not_double_known_not_int32(value, true);
        self.unbox_double_without_assertions(value, scratch, value_as_fpr);
        if invert {
            truthy.push(self.masm.branch_double_zero_or_nan(value_as_fpr, temp_fpr));
            done.push(self.masm.jump());
        } else {
            done.push(self.masm.branch_double_zero_or_nan(value_as_fpr, temp_fpr));
            truthy.push(self.masm.jump());
        }

        not_double.link(&mut self.masm);

        truthy.push(self.masm.branch64_imm64(
            if invert {
                RelationalCondition::NotEqual
            } else {
                RelationalCondition::Equal
            },
            value,
            unsafe { Value::true_().u.as_int64 },
        ));

        done.link(&mut self.masm);
        truthy
    }

    pub fn branch_if_truthy(
        &mut self,
        value: Reg,
        scratch: Reg,
        scratch2: Reg,
        scratch_fp0: FPReg,
        scratch_fp1: FPReg,
        should_check_masquerades_as_undefined: bool,
    ) -> JumpList {
        self.branch_if_value(
            value,
            scratch,
            scratch2,
            scratch_fp0,
            scratch_fp1,
            should_check_masquerades_as_undefined,
            false,
        )
    }
    pub fn branch_if_falsey(
        &mut self,
        value: Reg,
        scratch: Reg,
        scratch2: Reg,
        scratch_fp0: FPReg,
        scratch_fp1: FPReg,
        should_check_masquerades_as_undefined: bool,
    ) -> JumpList {
        self.branch_if_value(
            value,
            scratch,
            scratch2,
            scratch_fp0,
            scratch_fp1,
            should_check_masquerades_as_undefined,
            true,
        )
    }

    pub fn emit_jump_if_not_int(&mut self, reg1: Reg, reg2: Reg, scratch: Reg) -> Jump {
        self.masm.move_rr(reg1, scratch);
        self.masm.and64(reg2, scratch, scratch);
        self.branch_if_not_int32(scratch, true)
    }

    pub fn emit_jump_slow_case_if_not_ints(&mut self, reg1: Reg, reg2: Reg, scratch: Reg) {
        let j = self.emit_jump_if_not_int(reg1, reg2, scratch);
        self.add_slow_case(j)
    }
}

pub fn disasm_code(comments: Option<&HashMap<u32, String>>, code: *const u8, len: usize) {
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
        if let Some(comments) = comments {
            if let Some(c) = comments.get(&((i.address() - code as u64) as u32)) {
                println!("{}", c);
            }
        }
        println!("\t{}", i);
    }
}
