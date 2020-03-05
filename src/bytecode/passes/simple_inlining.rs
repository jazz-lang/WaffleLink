use super::*;
use crate::bytecode;
use crate::runtime;
use crate::util::arc::Arc;
use runtime::cell::*;
use runtime::module::*;
use std::collections::HashMap;
pub struct SimpleInliningPass {
    should_inline: HashMap<usize, bool>,
}

pub fn do_inlining(f: &mut Arc<Function>, module: &Arc<Module>) {
    let inlinable = get_constant_functions(module, &f.code).iter().filter(|(_,x)|
        !Arc::ptr_eq(f, x) // recursion, cannot inline
            && x.code.len() <= 6
            && !x.code.iter().any(|x| { // functions throws exception, we need to reorder catch table to inline function.
                x.instructions.iter().any(|x| if let Instruction::Throw(_) = x {
                    true
                } else {
                    false
                })
            })
    ).map(|(id,f)| (*id,f.clone())).collect::<HashMap<u16,Arc<Function>>>();

    let mut id = f.code.len();
    let mut new_blocks = vec![];
    let mut remap: HashMap<usize,usize> = HashMap::new();
    for bb in f.code.iter() {
        let mut curr_block = BasicBlock {
            index: new_blocks.len(),
            liveout: vec![],
            instructions: vec![]
        };

        for ins in bb.instructions.iter() {
            if let Instruction::Call(dest,callee,_) = ins {
                if let Some(func) = inlinable.get(&callee) {
                    remap.insert(bb.index,new_blocks.len());
                    curr_block.instructions.push(Instruction::Branch(new_blocks.len() as u16 + 1));
                    new_blocks.push(curr_block.clone());

                    let mut remap_inlined = HashMap::new();
                    let mut inlined = vec![];
                    let mut id = new_blocks.len();
                    for bb in func.code.iter() {
                        remap_inlined.insert(bb.index,new_blocks.len());
                        let new_bb = BasicBlock {
                            index: id,
                            liveout: vec![],
                            instructions: bb.instructions.clone()
                        };
                        id += 1;
                        inlined.push(new_bb);
                    }

                    for bb in inlined.iter_mut() {
                        for (from,to) in remap_inlined.iter() {
                            bb.try_replace_branch_targets(*from as _,*to as _);
                        }
                    }
                    let continue_bb_id = new_blocks.len() + inlined.len();
                    for bb in inlined.iter_mut() {
                        if bb.instructions.is_empty() {continue;}
                        if let Instruction::Return(Some(r)) = bb.instructions.last().copied().unwrap() {
                            /*bb.instructions.insert(bb.instructions.len() - 2,Instruction::Move(*dest,r));
                            *bb.instructions.last_mut().unwrap() = Instruction::Branch(continue_bb_id as _);*/
                            bb.instructions.pop();
                            bb.instructions.push(Instruction::Move(*dest,r));
                            bb.instructions.push(Instruction::Branch(continue_bb_id as _));
                        }
                    }
                    new_blocks.extend(inlined.into_iter());
                    curr_block = BasicBlock {
                        index: new_blocks.len(),
                        liveout: vec![],
                        instructions: vec![]
                    };


                }
            } else {
                curr_block.instructions.push(*ins);
            }
        }
        remap.insert(bb.index,new_blocks.len());
        new_blocks.push(curr_block);
    }

    for (from,to) in remap.iter() {
        for bb in new_blocks.iter_mut() {
            bb.try_replace_branch_targets(*from as u16,*to as u16);
        }
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
                        CellValue::Function(ref f) => {functions.insert(*reg, f.clone());},
                        _ => (),
                    }
                }
            }
        }
    }
    functions
}
