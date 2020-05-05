use crate::bytecode::*;
use crate::runtime;
use cgc::api::*;
use runtime::value::*;
use virtual_reg::*;
pub struct CallFrame {
    pub func: Value,
    pub registers: Vec<Value>,
    /// a.k.a arguments
    pub entries: Vec<Value>,
    pub this: Value,
    pub ip: usize,
    pub bp: usize,
    pub code: Handle<CodeBlock>,
}

impl CallFrame {
    pub fn r(&self, r: VirtualRegister) -> Value {
        if r.is_local() {
            self.registers[r.to_local() as usize]
        } else if r.is_argument() {
            self.entries[r.to_argument() as usize]
        } else if r.is_constant() {
            self.code.constants[r.to_constant() as usize]
        } else {
            unreachable!()
        }
    }

    pub fn r_mut(&mut self, r: VirtualRegister) -> &mut Value {
        if r.is_local() {
            &mut self.registers[r.to_local() as usize]
        } else if r.is_argument() {
            &mut self.entries[r.to_argument() as usize]
        } else if r.is_constant() {
            &mut self.code.constants[r.to_constant() as usize]
        } else {
            unreachable!()
        }
    }
}
