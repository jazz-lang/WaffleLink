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

use std::collections::HashSet;

/// This pass replaces all calls in tail position to tail calls.
/// Since Waffle interpreter does not support mutual recursion this step is necessary if we want
/// to handle recursion and get good execution speed.
pub struct TailCallEliminationPass;

use super::*;
use crate::bytecode;
use crate::util::arc::Arc;

impl BytecodePass for TailCallEliminationPass {
    fn execute(&mut self, f: &mut Arc<Function>) {
        let f = &mut f.code;
        let mut returns = HashSet::new();
        for bb in f.iter() {
            if bb.instructions.is_empty() {
                continue;
            }
            if let Instruction::Return(_) = bb.instructions.last().unwrap() {
                returns.insert(bb.index as u16);
            }
        }
        for bb in f.iter_mut() {
            // The last instruction is always a Return or Branch to Return instruction, so we check the
            // instruction that preceeds it.
            if bb.instructions.is_empty() || bb.instructions.len() < 2 {
                continue;
            }
            let pos = bb.instructions.len() - 2;

            let ins = bb.instructions.get(pos);
            if ins.is_none() {
                continue;
            } else {
                let ins = ins.copied().unwrap();
                if let Instruction::Call(dest, callee, argc) = ins {
                    if let Instruction::Return(_) = bb.instructions.last().unwrap() {
                        bb.instructions[pos] = Instruction::TailCall(dest, callee, argc);
                    } else {
                        let targets = bb.branch_targets();
                        if let Some(target) = targets.0 {
                            if returns.contains(&target) {
                                bb.instructions[pos] = Instruction::TailCall(dest, callee, argc);
                            }
                        }
                        if let Some(target) = targets.1 {
                            if returns.contains(&target) {
                                bb.instructions[pos] = Instruction::TailCall(dest, callee, argc);
                            }
                        }
                    }
                }
            }
        }
    }
}
