/// Bytecode the client gives us may contain several Return instructions. However,
/// internally we want a single exit point for a function. In this pass, we
/// create a return sink (a block), and rewrite all the Return instruction into
/// a Branch with return values.
///
/// NOTE: This pass should run after register allocation.
pub struct RetSink;
use super::*;
use crate::runtime::cell::Function;
use crate::util::arc::Arc;

impl BytecodePass for RetSink {
    fn execute(&mut self, f: &mut Arc<Function>) {
        let return_sink = { BasicBlock::new(vec![Instruction::Return(Some(0))], f.code.len()) };
        f.code.push(return_sink);
        let target = f.code.len() - 1;
        for block in f.code.iter_mut() {
            let mut new_instructions = vec![];
            for ins in block.instructions.iter() {
                match *ins {
                    Instruction::Return(Some(x)) => {
                        new_instructions.push(Instruction::Move(0, x));
                        new_instructions.push(Instruction::Branch(target as _));
                    }
                    Instruction::Return(None) => {
                        new_instructions.push(Instruction::LoadNull(0));
                        new_instructions.push(Instruction::Branch(target as _));
                    }
                    x => new_instructions.push(x),
                }
            }
            block.instructions = new_instructions;
        }
    }
}
