use super::*;
use crate::assembler;
use crate::bytecode;
use crate::interpreter::callstack::*;
use crate::jit::*;
use crate::runtime;
use assembler::cpu::*;
use assembler::masm::*;
use bytecode::{def::*, virtual_reg::*, *};
use crate::heap::api::*;
use func::*;
pub struct AddGenerator {
    pub ins: Ins,
    pub slow_path: Label,
    pub lhs: VirtualRegister,
    pub rhs: VirtualRegister,
    pub dst: VirtualRegister,
    pub end: Label,
}

impl FullGenerator for AddGenerator {
    fn fast_path(&mut self, gen: &mut FullCodegen) -> bool {
        let lhs = self.lhs;
        let rhs = self.rhs;
        let dst = self.dst;
        let slow_path = gen.masm.create_label();
        self.end = gen.masm.create_label();
        // load registers and move them to argument regs to call slow path when needed.

        gen.load_registers2(lhs, rhs, CCALL_REG_PARAMS[0], CCALL_REG_PARAMS[1]);
        gen.masm
            .emit_comment("if (!is_int32(lhs) || !is_int32(rhs) goto slow_path;");
        gen.masm
            .load_int_const(MachineMode::Int64, REG_RESULT, -562949953421312);
        gen.masm.asm.lea(
            RAX.into(),
            assembler::Address::offset(REG_RESULT.into(), -1),
        );
        gen.masm
            .cmp_reg(MachineMode::Int64, REG_RESULT, CCALL_REG_PARAMS[0]);
        gen.masm.jump_if(CondCode::UnsignedGreater, slow_path);
        gen.masm
            .cmp_reg(MachineMode::Int64, REG_RESULT, CCALL_REG_PARAMS[1]);
        gen.masm.jump_if(CondCode::UnsignedGreater, slow_path);
        gen.masm.int_add(
            MachineMode::Int32,
            REG_RESULT,
            CCALL_REG_PARAMS[0],
            CCALL_REG_PARAMS[1],
        );
        gen.masm.emit_comment("if (overflow) goto slow_path");
        // TODO: We do not have overflow condition in MASM, add it later.
        gen.masm
            .asm
            .setcc_r(assembler::Condition::Overflow, REG_TMP1.into());
        gen.masm.cmp_zero(MachineMode::Int8, REG_TMP1);

        gen.masm.jump_if(CondCode::Zero, slow_path);
        gen.masm.new_int(REG_RESULT, REG_RESULT);
        gen.store_register(dst);
        self.slow_path = slow_path;
        /*gen.masm.bind_label(slow_path);
        self.load_registers2(lhs, rhs, CCALL_REG_PARAMS[0], CCALL_REG_PARAMS[1]);
        gen.masm.raw_call(__add_slow_path as *const u8);
        self.store_register(dst);*/
        gen.masm.bind_label(self.end);
        true
    }
    fn slow_path(&mut self, gen: &mut FullCodegen) {
        gen.masm.emit_comment(format!("({}) slow_path:", self.ins));
        gen.masm.bind_label(self.slow_path);
        gen.load_registers2(self.lhs, self.rhs, CCALL_REG_PARAMS[0], CCALL_REG_PARAMS[1]);
        gen.masm.raw_call(__add_slow_path as *const u8);
        gen.store_register(self.dst);
        gen.masm.jump(self.end);
    }
}
