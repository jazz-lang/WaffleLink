use super::basicblock::*;
use super::instruction::*;
use crate::util::arc::Arc;

pub mod simplify;

pub trait BytecodePass {
    fn execute(&mut self, code: &Arc<Vec<BasicBlock>>);
}
