use super::basicblock::*;
use super::instruction::*;
use crate::util::arc::Arc;

pub mod peephole;
//pub mod regalloc;
pub mod ret_sink;
pub mod simplify;

pub trait BytecodePass {
    fn execute(&mut self, code: &mut Arc<Vec<BasicBlock>>);
}
