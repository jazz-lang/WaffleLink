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
pub struct ModGenerator {
    pub ins: Ins,
    pub slow_path: Label,
    pub lhs: VirtualRegister,
    pub rhs: VirtualRegister,
    pub dst: VirtualRegister,
    pub end: Label,
}

impl FullGenerator for ModGenerator {
    fn fast_path(&mut self, gen: &mut FullCodegen) -> bool {
        // TODO: Fast path: number % number, slow path: invoke __mod_slow_path.
        gen.load_registers2(self.lhs,self.rhs,CCALL_REG_PARAMS[0],CCALL_REG_PARAMS[1]);
        gen.masm.raw_call(__mod_slow_path as *const _);
        gen.store_register(self.dst);
        false
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

pub extern "C" fn __mod_slow_path(x: Value,y: Value) -> Value {
    Value::number(x.to_number() % y.to_number())
}