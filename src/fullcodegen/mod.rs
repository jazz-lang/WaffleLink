//! FullCodegen JIT
//!
//!
//! FullCodegen is baseline JIT compiler that emits unoptimized code.
pub mod generator;
pub mod jitadd_generator;

use crate::bytecode;
use crate::runtime;
use bytecode::{*,def::*,virtual_reg::*};
use runtime::value::*;
use runtime::cell::*;
use runtime::*;
use cgc::api::*;
use crate::assembler;
use assembler::masm::*;
use crate::jit::*;
use func::*;
use types::*;
use assembler::cpu::*;
use crate::interpreter::callstack::*;
pub struct FullCodegen {
    code: Handle<CodeBlock>,
    masm: MacroAssembler
}

impl FullCodegen {

    pub fn load_registers(&mut self,to: Reg) {
        self.masm.load_mem(MachineMode::Int64,AnyReg::Reg(to),Mem::Base(Reg::from(R14),offset_of!(CallFrame,registers) as _))
    }

    pub fn compile(&mut self) {


    }
}
