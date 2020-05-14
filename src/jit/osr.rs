use std::collections::HashMap;
pub struct OSR {
    /// Map from basic block id to label in assembly
    pub labels: HashMap<u32, OSREntry>,
}

/// OSR Entry used for entering specific blocks of code.
///
/// OSR might be used to enter some specific bytecode impl in interpreter
/// or some specific code in assembly.
///
#[repr(C)]
pub struct OSREntry {
    pub to_ip: u32,
    pub to_bp: u32,
}

impl OSREntry {
    pub fn jit_jmp_addr(&self) -> u32 {
        self.to_ip
    }

    pub fn ip(&self) -> u32 {
        self.to_ip
    }

    pub fn bp(&self) -> u32 {
        self.to_bp
    }
}

#[derive(Clone)]
pub struct OSRTable {
    pub labels: Vec<usize>,
}
use crate::interpreter::callstack::*;
pub struct OSRStub {
    pub code: Option<super::func::Code>,
}
use crate::runtime::Runtime;
impl OSRStub {
    pub fn new() -> Self {
        Self { code: None }
    }
    pub fn generate(&mut self, rt: &mut Runtime) {
        use super::types::*;
        use super::*;
        use crate::assembler::{cpu::*, masm::*, masmx64::*};
        let mut masm = MacroAssembler::new();
        masm.prolog();
        masm.copy_reg(MachineMode::Int64, REG_RESULT, CCALL_REG_PARAMS[3]);
        masm.copy_reg(MachineMode::Int64, REG_THREAD, CCALL_REG_PARAMS[0]);
        masm.copy_reg(MachineMode::Int64, REG_CALLFRAME, CCALL_REG_PARAMS[1]);
        masm.epilog_without_return();
        masm.jump_reg(REG_RESULT);
        self.code = Some(masm.jit(rt, 0, func::JitDescriptor::WaffleStub));
    }

    pub fn get(&self) -> extern "C" fn(&mut Runtime, &mut CallFrame, usize) -> super::JITResult {
        unsafe { std::mem::transmute(self.code.as_ref().unwrap().instruction_start()) }
    }
}
