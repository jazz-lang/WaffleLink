//! Various bytecode strength reduction passes.
//!
//!
//! Passes that run by default:
//! - Constant folding
//! - CFG simplifier - remove unecessary try/catch and unused basic blocks
//! - Dead constants elimination - delete unused constant value
//! - Load after store
//! - CSE - Replace common sub expressions (cse) with the previously defined one.
//!
//! Passes that run with -Ounsafe
//! - Inlining - tries to inline calls to functions in constant table.
//!
//!
//! WARNING: -Ounsafe enables very unsafe optimizations that may potentionally lead to
//! undefined behavior or runtime errors.

use crate::bytecode::*;
use crate::runtime::value::Value;
use cgc::api::Handle;
use def::*;
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
            _ => continue,
        }
    }
    false
}

impl ConstantFolding {
    pub fn run(mut code: Handle<CodeBlock>) {
        let mut constants = std::collections::HashMap::new();
        for bb in code.clone().code.iter_mut() {
            for i in 0..bb.code.len() {
                match bb.code[i] {
                    Ins::Mov { dst, src } => {
                        if src.is_constant() {
                            if is_constant(code.constants[src.to_constant() as usize]) {
                                constants.insert(dst, code.constants[src.to_constant() as usize]);
                            }
                        }
                    }
                    Ins::Add { dst, lhs, src, .. } => {
                        if let Some(lhs) = constants.get(&lhs) {
                            if let Some(rhs) = constants.get(&src) {
                                if lhs.is_int32() && rhs.is_int32() {
                                    let new_c = code.constants.len() as i32;
                                    code.constants
                                        .push(Value::new_int(lhs.as_int32() + rhs.to_int32()));
                                    bb.code[i] = Ins::Mov {
                                        dst,
                                        src: VirtualRegister::constant(new_c),
                                    };
                                } else {
                                    let new_c = code.constants.len() as i32;
                                    code.constants
                                        .push(Value::number(lhs.to_number() + rhs.to_number()));
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
                                    code.constants
                                        .push(Value::new_int(lhs.as_int32() - rhs.to_int32()));
                                    bb.code[i] = Ins::Mov {
                                        dst,
                                        src: VirtualRegister::constant(new_c),
                                    };
                                } else {
                                    let new_c = code.constants.len() as i32;
                                    code.constants
                                        .push(Value::number(lhs.to_number() - rhs.to_number()));
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
                                    code.constants
                                        .push(Value::new_int(lhs.as_int32() * rhs.to_int32()));
                                    bb.code[i] = Ins::Mov {
                                        dst,
                                        src: VirtualRegister::constant(new_c),
                                    };
                                } else {
                                    let new_c = code.constants.len() as i32;
                                    code.constants
                                        .push(Value::number(lhs.to_number() * rhs.to_number()));
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
                                code.constants
                                    .push(Value::number(lhs.to_number() / rhs.to_number()));
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
