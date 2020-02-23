use super::basicblock::*;
use super::instruction::*;
use crate::util::arc::Arc;

pub mod peephole;
pub mod regalloc;
pub mod ret_sink;
pub mod simplify;
use crate::runtime::cell::Function;
pub trait BytecodePass {
    fn execute(&mut self, f: &mut Arc<Function>);
}
