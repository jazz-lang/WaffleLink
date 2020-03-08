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

use super::*;
use crate::runtime::cell::Function;
use crate::util::arc::Arc;

use std::collections::HashMap;
// TODO: This pass does not work as it should.
pub struct CSEPass;

impl BytecodePass for CSEPass {
    fn execute(&mut self, f: &mut Arc<Function>) {
        let mut ins_map = HashMap::new();
        let mut stats = 0;
        for bb in f.code.iter_mut() {
            for i in bb.instructions.iter_mut() {
                let k = if let Instruction::Binary(_, _, lhs, rhs) = i {
                    *i
                } else {
                    match i {
                        Instruction::LoadInt { .. }
                        | Instruction::LoadNumber { .. }
                        | Instruction::LoadTrue { .. }
                        | Instruction::LoadFalse { .. }
                        | Instruction::LoadNull { .. }
                        | Instruction::LoadUndefined { .. } => *i,
                        _ => continue,
                    }
                };
                if ins_map.contains_key(&k) {
                    let ins_new = ins_map[&k];
                    *i = ins_new; // this is wrong, we should replace all `i` uses.
                    stats += 1;
                } else {
                    ins_map.insert(k, *i);
                }
            }
        }
        if stats > 0 {
            log::debug!("Replaced {} instructions", stats);
        }
    }
}
