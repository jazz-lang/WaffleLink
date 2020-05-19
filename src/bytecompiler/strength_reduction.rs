//! Various bytecode strength reduction passes.
//!
//!
//! Passes that run by default:
//! - Constant folding
//! - CFG simplifier - remove unecessary try/catch and unused basic blocks
//! - Dead constants elimination - delete unused constant value
//! - Load after store
//! - Local CSE - Replace common sub expressions (cse) with the previously defined one.
//!
//! Passes that run with -Ounsafe
//! - Inlining - tries to inline calls to functions in constant table.
//! - LICM
//! - GVN and Global CSE
//!
//! WARNING: -Ounsafe enables very unsafe optimizations that may potentionally lead to
//! undefined behavior or runtime errors.

use crate::bytecode::*;
use crate::runtime::value::Value;
use def::*;
use std::collections::HashMap;
use virtual_reg::*;
/// Constant folding pass.
///
///
/// This pass tries to execute some instructions before interpreting.
/// Example:
/// ```must_fail
/// mov loc0, id0
/// mov loc1, id1
/// add loc0,loc0,loc1
///
/// id0: Int32 = 3
/// id1: Int32 = 2
/// ```
/// Will become:
/// ```must_fail
/// mov loc0, id2
///
/// id0: Int32 = 3
/// id1: Int32 = 2
/// id2: Int32 = 5
/// ```
/// Notice that constant table is not cleaned, this is done in separate pass.
///
pub struct ConstantFolding;

fn is_constant(x: Value) -> bool {
    if x.is_boolean() {
        true
    } else if x.is_number() {
        true
    } else if x.is_undefined_or_null() {
        true
    } else {
        false
    }
}

#[allow(unused)]
/// Returns true if `x` instructions might throw exception.
fn possibly_throws(x: &[Ins]) -> bool {
    for ins in x {
        match ins {
            Ins::Throw { .. } => return true,
            Ins::Call { .. } => return true,
            Ins::GetById { .. } => return true,
            Ins::GetByVal { .. } => return true,
            Ins::IteratorNext { .. } => return true,
            Ins::IteratorOpen { .. } => return true,
            Ins::PutById { .. } => return true,
            Ins::PutByVal { .. } => return true,
            Ins::Await { .. } => return true,
            Ins::LoadGlobal { .. } => return true,
            Ins::Concat { .. } => return true,
            _ => continue,
        }
    }
    false
}

impl ConstantFolding {
    pub fn run(mut code: crate::Rc<CodeBlock>) {
        let mut constants = std::collections::HashMap::new();
        for (i, c) in code.constants.iter().enumerate() {
            if is_constant(*c.0) {
                constants.insert(VirtualRegister::constant(i as _), *c.0);
            }
        }
        for bb in code.clone().code.iter_mut() {
            for i in 0..bb.code.len() {
                match bb.code[i] {
                    Ins::Mov { dst, src } => {
                        if src.is_constant() {
                            let c = code.get_constant(src.to_constant());
                            if is_constant(c) {
                                constants.insert(dst, c);
                            }
                        }
                    }
                    Ins::Add { dst, lhs, src, .. } => {
                        if let Some(lhs) = constants.get(&lhs).copied() {
                            if let Some(rhs) = constants.get(&src).copied() {
                                if lhs.is_int32() && rhs.is_int32() {
                                    let c = Value::new_int(lhs.as_int32() + rhs.to_int32());
                                    let new_c = code.new_constant(c);
                                    bb.code[i] = Ins::Mov { dst, src: new_c };
                                    constants.insert(dst, c);
                                } else {
                                    let c = Value::number(lhs.to_number() + rhs.to_number());
                                    let new_c = code.new_constant(c);
                                    bb.code[i] = Ins::Mov { dst, src: new_c };
                                    constants.insert(dst, c);
                                }
                            }
                        }
                    }
                    Ins::Sub { dst, lhs, src, .. } => {
                        if let Some(lhs) = constants.get(&lhs) {
                            if let Some(rhs) = constants.get(&src) {
                                if lhs.is_int32() && rhs.is_int32() {
                                    let c = Value::new_int(lhs.as_int32() - rhs.to_int32());
                                    let new_c = code.new_constant(c);
                                    bb.code[i] = Ins::Mov { dst, src: new_c };
                                    constants.insert(dst, c);
                                } else {
                                    let c = Value::number(lhs.to_number() - rhs.to_number());
                                    let new_c = code.new_constant(c);
                                    bb.code[i] = Ins::Mov { dst, src: new_c };
                                    constants.insert(dst, c);
                                }
                            }
                        }
                    }
                    Ins::Mul { dst, lhs, src, .. } => {
                        if let Some(lhs) = constants.get(&lhs) {
                            if let Some(rhs) = constants.get(&src) {
                                if lhs.is_int32() && rhs.is_int32() {
                                    let c = Value::new_int(lhs.as_int32() * rhs.to_int32());
                                    let new_c = code.new_constant(c);
                                    bb.code[i] = Ins::Mov { dst, src: new_c };
                                    constants.insert(dst, c);
                                } else {
                                    let c = Value::number(lhs.to_number() * rhs.to_number());
                                    let new_c = code.new_constant(c);
                                    bb.code[i] = Ins::Mov { dst, src: new_c };
                                    constants.insert(dst, c);
                                }
                            }
                        }
                    }
                    Ins::Div { dst, lhs, src, .. } => {
                        if let Some(lhs) = constants.get(&lhs) {
                            if let Some(rhs) = constants.get(&src) {
                                let c = Value::number(lhs.to_number() / rhs.to_number());
                                let new_c = code.new_constant(c);
                                bb.code[i] = Ins::Mov { dst, src: new_c };
                                constants.insert(dst, c);
                            }
                        }
                    }
                    Ins::JumpConditional {
                        cond,
                        if_true,
                        if_false,
                    } => {
                        if let Some(c) = constants.get(&cond) {
                            if c.to_boolean() {
                                bb.code[i] = Ins::Jump { dst: if_true };
                            } else {
                                bb.code[i] = Ins::Jump { dst: if_false };
                            }
                        }
                    }

                    _ => (),
                }
            }
        }
    }
}

/// Common subexpression elimination (CSE) is a compiler optimization that searches for instances of identical expressions
/// (i.e., they all evaluate to the same value), and analyzes whether it is worthwhile replacing them with a single variable
/// holding the computed value.
///
/// Example:
/// ```must_fail
/// add loc0,arg0,arg1
/// mov loc1,id1
/// add loc0,arg0,arg1
/// add loc2,loc1,loc0
/// ```
/// Will become:
/// ```must_fail
/// add loc0,arg0,arg1
/// mov loc1,id1
/// add loc2,loc1,loc0
/// ```
pub struct LocalCSE;

impl LocalCSE {
    pub fn run(code: crate::Rc<CodeBlock>) {
        //return;
        for bb in code.clone().code.iter_mut() {
            let mut map = HashMap::new();
            for i in 0..bb.code.len() {
                let ins = bb.code[i];
                let k = if let Some((x, op, y)) = ins.to_binary() {
                    (x, op, y)
                } else {
                    continue;
                };
                if map.contains_key(&k) {
                    let def = ins.get_defs()[0];
                    let ins_new: Ins = map[&k];
                    let res = ins_new.get_defs()[0];
                    bb.code[i] = Ins::Mov { dst: def, src: res };
                } else {
                    map.insert(k, ins);
                }
            }
        }
    }
}

/// Glue blocks together if a block has only one predecessor.
///
///
///  Remove blocks with a single jump in it.
///  code:
/// ```must_fail
///            jump A
///            A:
///            jump B
///            B:
/// ```
/// Transforms into:
/// ```must_fail
///            jump B
///            B:
/// ```
///
///
/// We have three easy simplification rules:
///
/// 1) If a successor is a block that just jumps to another block, then jump directly to
///    that block.
///
/// 2) If all successors are the same and the operation has no effects, then use a jump
///    instead.
///
/// 3) If you jump to a block that is not you and has one predecessor, then merge.
///
/// Note that because of the first rule, this phase may introduce critical edges. That's fine.
/// If you need broken critical edges, then you have to break them yourself.

pub struct CleanPass;

impl CleanPass {
    fn find_empty_blocks(code: &CodeBlock) -> Vec<u32> {
        let mut empty_blocks = vec![];
        for bb in code.code.iter().skip(1) {
            if bb.last().is_jump() && bb.size() == 1 {
                empty_blocks.push(bb.id);
            }
        }
        return empty_blocks;
    }
    /// Remove empty basic blocks from function.
    fn remove_empty_blocks(code: crate::Rc<CodeBlock>) {
        const VERBOSE: bool = false;
        log::trace!("Code before removing empty blocks:");
        code.trace_code(true);
        let mut old_ids = HashMap::new();
        for bb in code.code.iter() {
            old_ids.insert(bb.id, bb.id);
        }
        let empty = Self::find_empty_blocks(&*code);
        //let mut stat = 0;
        for &block in empty.iter() {
            let preds = code.cfg.as_ref().unwrap().get_preds(&block);
            let _succs = code.cfg.as_ref().unwrap().get_succs(&block);
            // Do not remove if preceeded by itself:
            if preds.contains(&block) {
                continue;
            }

            let tgt = code.code[block as usize].branch_targets()[0];
            let mut c = code.clone();
            for pred in preds.iter() {
                trace_if!(VERBOSE, "Replace %{}->%{}", block, tgt);
                c.code[*pred as usize].try_replace_branch_targets(block, tgt);
            }
        }
        log::trace!("After: ");
        code.trace_code(true);
    }

    pub fn remove_dead_blocks(code: crate::Rc<CodeBlock>) {
        use crate::common::bitvec::BitVector;
        let mut seen = BitVector::new(code.code.len());
        seen.insert(0);
        let mut worklist = Vec::with_capacity(4);
        worklist.push(0);
        while let Some(bb) = worklist.pop() {
            for succ in code.cfg.as_ref().unwrap().get_succs(&bb) {
                if seen.insert(*succ as usize) {
                    worklist.push(*succ);
                }
            }
        }
        Self::retain_basic_blocks(code, &seen, false)
    }

    fn retain_basic_blocks(
        mut code: crate::Rc<CodeBlock>,
        keep: &crate::common::bitvec::BitVector,
        _x: bool,
    ) {
        const VERBOSE: bool = false;
        let num_blocks = code.code.len();
        let mut replacements: Vec<_> = (0..num_blocks as u32).map(|_| 0).collect();
        let mut used_blocks = 0;
        for alive_index in keep.iter() {
            let alive_index: u32 = alive_index as _;
            replacements[alive_index as usize] = used_blocks;
            if alive_index != used_blocks {
                trace_if!(VERBOSE, "Swap %{} and %{}", alive_index, used_blocks);
                code.code[alive_index as usize].id = used_blocks as _;
                code.code.swap(alive_index as usize, used_blocks as usize);
            }
            used_blocks += 1;
        }
        code.code.truncate(used_blocks as _);
        for bb in code.code.iter_mut() {
            for target in bb.branch_targets() {
                bb.try_replace_branch_targets(target, replacements[target as usize]);
            }
        }
    }
    pub fn run(code: crate::Rc<CodeBlock>) {
        Self::remove_empty_blocks(code);
    }
}

use indexmap::IndexMap;

pub struct DCE;

use super::interference_graph::InterferenceGraph;

impl DCE {}

pub fn regalloc_and_reduce_strength(
    mut code: crate::Rc<CodeBlock>,
    _rt: &mut crate::runtime::Runtime,
) {
    code.cfg = Some(Box::new(build_cfg_for_code(&code.code)));
    // replace jumps
    CleanPass::remove_empty_blocks(code.clone());
    code.cfg = Some(Box::new(build_cfg_for_code(&code.code)));
    // delete unused blocks
    CleanPass::remove_dead_blocks(code.clone());
    code.cfg = Some(Box::new(build_cfg_for_code(&code.code)));

    LocalCSE::run(code.clone());
    //ConstantFolding::run(code);

    super::loopanalysis::loopanalysis(code.clone());
    let ra = super::graph_coloring::GraphColoring::start(
        code.clone(),
        &code.loopanalysis.as_ref().unwrap(),
    );

    for (temp, real) in ra.get_assignments() {
        for bb in code.code.iter_mut() {
            for ins in bb.code.iter_mut() {
                ins.replace_reg(temp, real);
            }
        }
    }

    code.code.iter_mut().for_each(|bb| {
        bb.code.retain(|ins| {
            if let Ins::Mov { dst, src } = ins {
                if dst == src {
                    return false;
                }
            }
            true
        })
    });
}
