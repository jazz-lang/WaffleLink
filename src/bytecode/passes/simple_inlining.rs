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
use crate::bytecode;
use crate::runtime;
use crate::util::arc::Arc;
use runtime::cell::*;
use runtime::module::*;
use std::collections::{HashMap, HashSet};
pub struct SimpleInliningPass {
    should_inline: HashMap<usize, bool>,
}

pub fn do_inlining(f: &mut Arc<Function>, module: &Arc<Module>) {
    let inlinable = get_constant_functions(module, &f.code)
        .iter()
        .filter(|(_, x)| {
            !Arc::ptr_eq(f, x) // recursion, cannot inline
            && x.code.len() <= 6
            && !x.code.iter().any(|x| { // functions throws exception, we need to reorder catch table to inline function.
                x.instructions.iter().any(|x| if let Instruction::Throw(_) = x {
                    true
                } else {
                    false
                })
            })
        })
        .map(|(id, f)| (*id, f.clone()))
        .collect::<HashMap<u16, Arc<Function>>>();
    let mut new_blocks = vec![];
    let mut id: usize = 0;
    for block in f.code.iter() {
        let mut cur_block = block.clone();
        cur_block.instructions.clear();
        let block = block.clone();
        for inst in block.instructions.iter() {
            if let Instruction::Call(dest, callee, _) = inst {
                if let Some(inlined_func) = inlinable.get(callee as _) {
                    log::debug!("Replace call with branch to {}", id);

                    let branch = Instruction::Branch(id as u16);
                    id += 1;
                    cur_block.instructions.push(branch);
                    cur_block.index = id - 1;
                    //id += 1;
                    new_blocks.push(cur_block.clone());
                    let old_name = cur_block.index;
                    let new_name = id;
                    id += 1;
                    cur_block = BasicBlock::new(vec![], new_name as _);

                    copy_inline_blocks(
                        &mut new_blocks,
                        cur_block.index,
                        inlined_func,
                        new_name as _,
                        &mut id,
                        *dest as _,
                    );
                }
            } else {
                cur_block.instructions.push(inst.clone());
            }
        }

        cur_block.index = id;
        new_blocks.push(cur_block);
        id += 1;
    }
    *f.code = new_blocks;
}

pub fn get_constant_functions(
    module: &Arc<Module>,
    code: &Arc<Vec<BasicBlock>>,
) -> HashMap<u16, Arc<Function>> {
    let mut functions = HashMap::new();
    for block in code.iter() {
        for ins in block.instructions.iter() {
            if let Instruction::LoadConst(reg, constant) = ins {
                let global = module.get_global_at(*constant as usize);
                if global.is_cell() {
                    match global.as_cell().get().value {
                        CellValue::Function(ref f) => {
                            functions.insert(*reg, f.clone());
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    functions
}

fn copy_inline_blocks(
    caller: &mut Vec<BasicBlock>,
    mut ret_block: usize,
    callee: &Arc<Function>,
    entry_block: usize,
    id: &mut usize,
    dest: usize,
) {
    let mut block_map = HashMap::new();
    for (i, block) in callee.code.iter().enumerate() {
        if i == 0 {
            block_map.insert(block.index, entry_block);
        } else {
            block_map.insert(block.index, *id);
            *id += 1;
        }
    }

    let fix_dest = |dest: usize| block_map.get(&dest).unwrap().clone();
    for old_block in callee.code.iter() {
        let old_id = old_block.index;
        let new_id = block_map.get(&old_id).copied().unwrap();
        let mut block = BasicBlock::new(vec![], new_id);
        {
            let old_block_content = &old_block.instructions;
            let block_content = &mut block.instructions;
            // Copy the old_block contents (minus the last one)
            for i in 0..old_block_content.len() - 1 {
                block_content.push(old_block_content[i].clone());
            }
            // check its last instruction
            let last_inst = *old_block_content.last().unwrap();
            match last_inst {
                Instruction::Return(Some(r)) => {
                    block_content.push(Instruction::Move(dest as _, r));
                    block_content.push(Instruction::Branch(ret_block as _));
                }
                Instruction::Branch(to) => {
                    block_content.push(Instruction::Branch(fix_dest(to as _) as _));
                }
                Instruction::BranchIfFalse(r, to) => {
                    block_content.push(Instruction::BranchIfFalse(r, fix_dest(to as _) as _));
                }
                Instruction::BranchIfTrue(r, to) => {
                    block_content.push(Instruction::BranchIfTrue(r, fix_dest(to as _) as _));
                }
                Instruction::ConditionalBranch(r, ift, iff) => {
                    block_content.push(Instruction::ConditionalBranch(
                        r,
                        fix_dest(ift as _) as _,
                        fix_dest(iff as _) as _,
                    ));
                }
                // last instruction is always branch or return
                _ => unreachable!(),
            }
        }
        caller.push(block);
    }
}
