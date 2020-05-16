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
pub struct DivGenerator {
    pub ins: Ins,
    pub slow_path: Label,
    pub lhs: VirtualRegister,
    pub rhs: VirtualRegister,
    pub dst: VirtualRegister,
    pub end: Label,
}

impl FullGenerator for DivGenerator {
    fn fast_path(&mut self, gen: &mut FullCodegen) -> bool {

        self.slow_path = gen.masm.create_label();
        self.end = gen.masm.create_label();
        gen.load_registers2(self.lhs,self.rhs,CCALL_REG_PARAMS[0],CCALL_REG_PARAMS[1]);
        gen.masm.emit_comment("if not_number(lhs) && not_number(rhs) goto slow_path");
        gen.masm.jmp_is_number(self.slow_path,CCALL_REG_PARAMS[0]);
        gen.masm.jmp_is_number(self.slow_path,CCALL_REG_PARAMS[1]);
        let not_int = gen.masm.create_label();
        let skip = gen.masm.create_label();
        gen.masm.jmp_nis_int32(not_int,CCALL_REG_PARAMS[0]);
        gen.masm.cvt_int32_to_double(CCALL_REG_PARAMS[0],XMM0);
        gen.masm.jump(skip);
        gen.masm.bind_label(not_int);
        gen.masm.as_double(CCALL_REG_PARAMS[0],XMM0);
        gen.masm.bind_label(skip);
        let not_int = gen.masm.create_label();
        let skip = gen.masm.create_label();
        gen.masm.jmp_nis_int32(not_int,CCALL_REG_PARAMS[1]);
        gen.masm.cvt_int32_to_double(CCALL_REG_PARAMS[1],XMM1);
        gen.masm.jump(skip);
        gen.masm.bind_label(not_int);
        gen.masm.as_double(CCALL_REG_PARAMS[1],XMM1);
        gen.masm.bind_label(skip);
        gen.masm.float_div(MachineMode::Float64,XMM0,XMM0,XMM1);
        gen.masm.new_number(XMM0);
        gen.store_register(self.dst);
        gen.masm.bind_label(self.end);
        true
    }
    fn slow_path(&mut self, gen: &mut FullCodegen) {
        gen.masm.emit_comment(format!("({}) slow_path:", self.ins));
        gen.masm.bind_label(self.slow_path);
        gen.load_registers2(self.lhs, self.rhs, CCALL_REG_PARAMS[0], CCALL_REG_PARAMS[1]);
        gen.masm.raw_call(__div_slow_path as *const u8);
        gen.store_register(self.dst);
        gen.masm.jump(self.end);
    }
}

pub extern "C" fn __div_slow_path(x: Value,y: Value) -> Value {
    Value::number(x.to_number() / y.to_number())
}