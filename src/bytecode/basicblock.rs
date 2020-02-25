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

use super::instruction::*;
use std::vec::Vec;
#[derive(Clone)]
pub struct BasicBlock {
    pub index: usize,

    pub instructions: Vec<Instruction>,
}

impl BasicBlock {
    pub fn new(ins: Vec<Instruction>, idx: usize) -> Self {
        Self {
            instructions: ins,
            index: idx,
        }
    }

    pub fn join(&mut self, other: BasicBlock) {
        self.instructions.pop();
        for ins in other.instructions {
            self.instructions.push(ins);
        }
    }

    pub fn try_replace_branch_targets(&mut self, to: u16, from: u16) -> bool {
        if self.instructions.is_empty() {
            return false;
        }
        let last_ins_id = self.instructions.len() - 1;
        let last_ins = &mut self.instructions[last_ins_id];
        match *last_ins {
            Instruction::ConditionalBranch(r, if_true, if_false) => {
                if if_true == from || if_false == from {
                    let if_true = if if_true == from { to } else { if_true };
                    let if_false = if if_false == from { to } else { if_false };
                    *last_ins = Instruction::ConditionalBranch(r, if_true, if_false);
                    true
                } else {
                    false
                }
            }
            Instruction::Branch(t) => {
                if t == from {
                    *last_ins = Instruction::Branch(to);
                    true
                } else {
                    false
                }
            }
            Instruction::BranchIfFalse(r, t) => {
                if t == from {
                    *last_ins = Instruction::BranchIfFalse(r, to);
                    true
                } else {
                    false
                }
            }
            Instruction::BranchIfTrue(r, t) => {
                if t == from {
                    *last_ins = Instruction::BranchIfTrue(r, to);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn branch_targets(&self) -> (Option<u16>, Option<u16>) {
        if self.instructions.is_empty() {
            return (None, None);
        }

        let last_ins = &self.instructions[self.instructions.len() - 1];
        match *last_ins {
            Instruction::ConditionalBranch(_, if_true, if_false) => (Some(if_true), Some(if_false)),
            Instruction::Branch(t)
            | Instruction::BranchIfFalse(_, t)
            | Instruction::BranchIfTrue(_, t) => (Some(t), None),
            Instruction::Return(_) => (None, None),
            _ => panic!("Terminator not found"),
        }
    }
}
use core::hash::{Hash, Hasher};

impl Hash for BasicBlock {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}
