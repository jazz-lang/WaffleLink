/*
*   Copyright (c) 2020 Adel Prokurov
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/

/// Bytecode the client gives us may contain several Return instructions. However,
/// internally we want a single exit point for a function. In this pass, we
/// create a return sink (a block), and rewrite all the Return instruction into
/// a Branch with return values.
pub struct RetSink;
use super::*;
use crate::runtime::cell::Function;
use crate::util::arc::Arc;

impl BytecodePass for RetSink {
    fn execute(&mut self, f: &mut Arc<Vec<BasicBlock>>) {
        let return_sink = { BasicBlock::new(vec![Instruction::Return(Some(0))], f.len()) };
        let target = f.len();
        for block in f.iter_mut() {
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
        f.push(return_sink);
    }
}
