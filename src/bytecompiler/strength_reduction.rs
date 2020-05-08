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
use cgc::api::Handle;
use def::*;
use std::collections::HashMap;
use virtual_reg::*;
/// Constant folding pass.
///
///
/// This pass tries to execute some instructions before interpreting.
/// Example:
/// ```
/// mov loc0, id0
/// mov loc1, id1
/// add loc0,loc0,loc1
///
/// id0: Int32 = 3
/// id1: Int32 = 2
/// ```
/// Will become:
/// ```
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
    pub fn run(mut code: Handle<CodeBlock>) {
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
                        if let Some(lhs) = constants.get(&lhs) {
                            if let Some(rhs) = constants.get(&src) {
                                if lhs.is_int32() && rhs.is_int32() {
                                    let new_c = code.constants.len() as i32;
                                    code.new_constant(Value::new_int(
                                        lhs.as_int32() + rhs.to_int32(),
                                    ));
                                    bb.code[i] = Ins::Mov {
                                        dst,
                                        src: VirtualRegister::constant(new_c),
                                    };
                                } else {
                                    let new_c = code.constants.len() as i32;
                                    code.new_constant(Value::number(
                                        lhs.to_number() + rhs.to_number(),
                                    ));
                                    bb.code[i] = Ins::Mov {
                                        dst,
                                        src: VirtualRegister::constant(new_c),
                                    };
                                }
                            }
                        }
                    }
                    Ins::Sub { dst, lhs, src, .. } => {
                        if let Some(lhs) = constants.get(&lhs) {
                            if let Some(rhs) = constants.get(&src) {
                                if lhs.is_int32() && rhs.is_int32() {
                                    let new_c = code.constants.len() as i32;
                                    code.new_constant(Value::new_int(
                                        lhs.as_int32() - rhs.to_int32(),
                                    ));
                                    bb.code[i] = Ins::Mov {
                                        dst,
                                        src: VirtualRegister::constant(new_c),
                                    };
                                } else {
                                    let new_c = code.constants.len() as i32;
                                    code.new_constant(Value::number(
                                        lhs.to_number() - rhs.to_number(),
                                    ));
                                    bb.code[i] = Ins::Mov {
                                        dst,
                                        src: VirtualRegister::constant(new_c),
                                    };
                                }
                            }
                        }
                    }
                    Ins::Mul { dst, lhs, src, .. } => {
                        if let Some(lhs) = constants.get(&lhs) {
                            if let Some(rhs) = constants.get(&src) {
                                if lhs.is_int32() && rhs.is_int32() {
                                    let new_c = code.constants.len() as i32;
                                    code.new_constant(Value::new_int(
                                        lhs.as_int32() * rhs.to_int32(),
                                    ));
                                    bb.code[i] = Ins::Mov {
                                        dst,
                                        src: VirtualRegister::constant(new_c),
                                    };
                                } else {
                                    let new_c = code.constants.len() as i32;
                                    code.new_constant(Value::number(
                                        lhs.to_number() * rhs.to_number(),
                                    ));
                                    bb.code[i] = Ins::Mov {
                                        dst,
                                        src: VirtualRegister::constant(new_c),
                                    };
                                }
                            }
                        }
                    }
                    Ins::Div { dst, lhs, src, .. } => {
                        if let Some(lhs) = constants.get(&lhs) {
                            if let Some(rhs) = constants.get(&src) {
                                let new_c = code.constants.len() as i32;
                                code.new_constant(Value::number(lhs.to_number() / rhs.to_number()));
                                bb.code[i] = Ins::Mov {
                                    dst,
                                    src: VirtualRegister::constant(new_c),
                                };
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
/// ```
/// add loc0,arg0,arg1
/// mov loc1,id1
/// add loc0,arg0,arg1
/// add loc2,loc1,loc0
/// ```
/// Will become:
/// ```
/// add loc0,arg0,arg1
/// mov loc1,id1
/// add loc2,loc1,loc0
/// ```
pub struct LocalCSE;

impl LocalCSE {
    pub fn run(code: Handle<CodeBlock>) {
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

pub fn regalloc_and_reduce_strength(mut code: Handle<CodeBlock>) {
    ConstantFolding::run(code);
    LocalCSE::run(code);
    code.cfg = Some(build_cfg_for_code(&code.code));
    super::loopanalysis::loopanalysis(code);

    let ra =
        super::graph_coloring::GraphColoring::start(code, &code.loopanalysis.as_ref().unwrap());
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
