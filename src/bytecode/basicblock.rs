use super::instruction::*;
use std::vec::Vec;
pub struct BasicBlock {
    pub index: usize,
    pub predecessors: Vec<usize>,
    pub successors: Vec<usize>,
    pub instructions: Vec<Instruction>,
}

impl BasicBlock {
    pub fn new(ins: Vec<Instruction>, idx: usize) -> Self {
        Self {
            instructions: ins,
            index: idx,
            successors: vec![],
            predecessors: vec![],
        }
    }
}
use core::hash::{Hash, Hasher};

impl Hash for BasicBlock {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}
