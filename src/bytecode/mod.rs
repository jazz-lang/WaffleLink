pub mod def;
pub mod virtual_reg;
// Register numbers used in bytecode operations have different meaning according to their ranges:
//      0x80000000-0xFFFFFFFF  Negative indices from the CallFrame pointer are entries in the call frame.
//      0x00000000-0x3FFFFFFF  Forwards indices from the CallFrame pointer are local vars and temporaries with the function's callframe.
//      0x40000000-0x7FFFFFFF  Positive indices from 0x40000000 specify entries in the constant pool on the CodeBlock.
pub const FIRST_CONSTNAT_REG_INDEX: i32 = 0x40000000;

use def::*;

pub struct BasicBlock {
    pub id: u32,
    pub code: Vec<def::Ins>,
}

impl BasicBlock {
    pub fn new(id: u32) -> Self {
        Self { id, code: vec![] }
    }
}

use cgc::api::{Finalizer, Traceable, Tracer};

impl Finalizer for BasicBlock {
    fn finalize(&mut self) {}
}

impl Traceable for BasicBlock {
    fn trace_with(&self, _tracer: &mut Tracer) {}
}
use crate::runtime::value::*;
use crate::runtime::*;
pub struct CodeBlock {
    pub constants: Vec<Value>,
    pub arg_regs_count: u32,
    pub tmp_regs_count: u32,
    pub code: Vec<BasicBlock>,
    pub hotness: usize,
    pub jit_stub: Option<extern "C" fn(&mut Runtime, Value, &[Value]) -> Result<Value, Value>>,
}

impl CodeBlock {
    pub fn dump<W: std::fmt::Write>(&self, b: &mut W) -> std::fmt::Result {
        for bb in self.code.iter() {
            writeln!(b, "%{}: ", bb.id)?;
            for (i, ins) in bb.code.iter().enumerate() {
                writeln!(b, "  [{:04}] {}", i, ins)?;
            }
        }
        Ok(())
    }
}

impl Traceable for CodeBlock {
    fn trace_with(&self, tracer: &mut Tracer) {
        self.constants.trace_with(tracer);
    }
}

impl Finalizer for CodeBlock {}
