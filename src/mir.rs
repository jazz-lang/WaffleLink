//! # MIR: Mid-level IR
//! We use MIR in optimizing and tracing JIT, MIR includes a few Waffle specific optimizations and it is lowered to LIR or MacroAssembler directly.

pub mod basic_block;
pub mod node;
pub mod opcodes;

pub struct MIRGraph {
    pub basic_blocks: Vec<basic_block::BasicBlock>,
    pub values: Vec<ValueData>,
    pub func_signatures: Vec<(Vec<Type>, Vec<Type>)>,
    current_bb: u32,
}

impl MIRGraph {
    pub fn walk_value_uses(&self, value: u32, mut f: impl FnMut((u32, u32))) {
        for i in 0..self.values[value as usize].uses.len() {
            f(self.values[value as usize].uses[i]);
        }
    }
}

pub struct ValueData {
    pub ty: Type,
    pub uses: Vec<(u32, u32)>,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Type {
    ValueI32,
    ValueNum,
    ValueAnyNum,
    ValueString,
    ValueArray,
    ValueUndefOrNull,
    ValueObject,
    ValueUnknown,
    I32,
    I64,
    I8,
    I16,

    F32,
    F64,

    Func(u32),

    Unknown,
}
