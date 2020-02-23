pub struct PeepholePass;

use super::*;
use crate::util::arc::Arc;

impl BytecodePass for PeepholePass {
    fn execute(&mut self, code: &mut Arc<Vec<BasicBlock>>) {
        for block in code.iter_mut() {
            for i in 0..block.instructions.len() {
                if let Instruction::Move(to, from) = block.instructions[i] {
                    if to == from {
                        // if two sides of a move instruction are the same,
                        // it is redundant, and can be eliminated
                        block.instructions.remove(i);
                    }
                }
            }
        }
    }
}
