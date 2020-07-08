use crate::value::Value;
use std::collections::HashMap;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod jit_x86;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use jit_x86::*;
#[cfg(target_pointer_width = "64")]
pub mod jit64;
#[cfg(target_pointer_width = "64")]
pub mod tail_call64;
use crate::builtins::WResult;
use crate::bytecode::*;
use crate::interpreter::callframe::CallFrame;
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

pub struct SlowCaseEntry {
    pub from: Jump,
    pub to: u32,
}

pub struct CallCompilationInfo {
    pub hot_path_begin: DataLabelPtr,
    pub hot_path_other: masm::Call,
    pub call_return_location: masm::Call,
}

impl<'a> JIT<'a> {
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
        let frame_top_offset = self.stack_pointer_offset_for(self.code_block);
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
    fn private_compile_slow_cases(&mut self) {
        /*for case in self.slow_cases.iter() {
            self.bytecode_index = case.to as _;
            // TODO
        }*/
    }
    pub fn link(&mut self) {
        let patch_buffer = &mut self.link_buffer;
        while let Some(record) = self.calls.pop() {
            if record.callee != 0 {
                patch_buffer.link_call(record.from, record.callee as *const _);
            }
        }
        let mut code_map = HashMap::new();
        for off in 0..self.labels.len() {
            if self.labels[off].asm_label().is_set() {
                code_map.insert(
                    off,
                    patch_buffer.location_of_label(self.labels[off].asm_label()),
                );
            }
        }
    }
}
