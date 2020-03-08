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
use crate::runtime::cell::*;
use crate::runtime::value::*;
use crate::util::arc::Arc;
use std::collections::HashMap;
pub struct ConstantFoldingPass;

impl BytecodePass for ConstantFoldingPass {
    fn execute(&mut self, f: &mut Arc<Function>) {
        let mut constants: HashMap<u16, Value> = HashMap::new();
        for bb in f.code.iter_mut() {
            for ins in bb.instructions.iter_mut() {
                match ins {
                    Instruction::Binary(op, dest, lhs, rhs) => {
                        match (constants.get(&lhs), constants.get(rhs)) {
                            (Some(lhs), Some(rhs)) => {
                                let c = match op {
                                    BinOp::Add => {
                                        if lhs.is_number() && rhs.is_number() {
                                            Value::new_double(lhs.to_number() + rhs.to_number())
                                        } else {
                                            continue;
                                        }
                                    }
                                    BinOp::Sub => {
                                        if lhs.is_number() && rhs.is_number() {
                                            Value::new_double(lhs.to_number() - rhs.to_number())
                                        } else {
                                            continue;
                                        }
                                    }
                                    BinOp::Div => {
                                        if lhs.is_number() && rhs.is_number() {
                                            Value::new_double(lhs.to_number() / rhs.to_number())
                                        } else {
                                            continue;
                                        }
                                    }
                                    BinOp::Mul => {
                                        if lhs.is_number() && rhs.is_number() {
                                            Value::new_double(lhs.to_number() * rhs.to_number())
                                        } else {
                                            continue;
                                        }
                                    }
                                    BinOp::Mod => {
                                        if lhs.is_number() && rhs.is_number() {
                                            Value::new_double(lhs.to_number() % rhs.to_number())
                                        } else {
                                            continue;
                                        }
                                    }
                                    BinOp::Equal => Value::from(lhs == rhs),
                                    BinOp::Greater => {
                                        if lhs.is_number() && rhs.is_number() {
                                            Value::from(lhs.to_number() > rhs.to_number())
                                        } else {
                                            continue;
                                        }
                                    }
                                    BinOp::Less => {
                                        if lhs.is_number() && rhs.is_number() {
                                            Value::from(lhs.to_number() < rhs.to_number())
                                        } else {
                                            continue;
                                        }
                                    }
                                    BinOp::LessOrEqual => {
                                        if lhs.is_number() && rhs.is_number() {
                                            Value::from(lhs.to_number() <= rhs.to_number())
                                        } else {
                                            continue;
                                        }
                                    }
                                    BinOp::GreaterOrEqual => {
                                        if lhs.is_number() && rhs.is_number() {
                                            Value::from(lhs.to_number() >= rhs.to_number())
                                        } else {
                                            continue;
                                        }
                                    }
                                    BinOp::NotEqual => Value::from(lhs != rhs),
                                    BinOp::Xor => {
                                        if lhs.is_number() && rhs.is_number() {
                                            Value::new_double(
                                                (lhs.to_number().floor() as i64
                                                    ^ rhs.to_number().floor() as i64)
                                                    as f64,
                                            )
                                        } else if lhs.is_bool() && rhs.is_bool() {
                                            Value::from(lhs.to_boolean() ^ rhs.to_boolean())
                                        } else {
                                            continue;
                                        }
                                    }
                                    BinOp::Or => {
                                        if lhs.is_number() && rhs.is_number() {
                                            Value::new_double(
                                                (lhs.to_number().floor() as i64
                                                    | rhs.to_number().floor() as i64)
                                                    as f64,
                                            )
                                        } else if lhs.is_bool() && rhs.is_bool() {
                                            Value::from(lhs.to_boolean() | rhs.to_boolean())
                                        } else {
                                            continue;
                                        }
                                    }
                                    BinOp::And => {
                                        if lhs.is_number() && rhs.is_number() {
                                            Value::new_double(
                                                (lhs.to_number().floor() as i64
                                                    & rhs.to_number().floor() as i64)
                                                    as f64,
                                            )
                                        } else if lhs.is_bool() && rhs.is_bool() {
                                            Value::from(lhs.to_boolean() & rhs.to_boolean())
                                        } else {
                                            continue;
                                        }
                                    }
                                    _ => continue,
                                };
                                constants.insert(*dest, c);
                                let new_ins = ins_for_const(c, *dest);
                                *ins = new_ins;
                            }
                            _ => continue,
                        }
                    }
                    Instruction::Unary(op, dest, r) => {
                        if let Some(val) = constants.get(&r) {
                            let c = match op {
                                UnaryOp::Not => {
                                    if val.is_number() {
                                        Value::new_double(
                                            (!(val.to_number().floor() as i64)) as f64,
                                        )
                                    } else {
                                        Value::from(!val.to_boolean())
                                    }
                                }
                                UnaryOp::Neg => Value::new_double(-val.to_number()),
                            };
                            constants.insert(*dest, c);
                            let new_ins = ins_for_const(c, *dest);
                            *ins = new_ins;
                        }
                    }
                    Instruction::LoadInt(r, val) => {
                        constants.insert(*r, Value::new_int(*val));
                    }
                    Instruction::LoadNumber(r, val) => {
                        constants.insert(*r, Value::new_double(f64::from_bits(*val)));
                    }
                    Instruction::LoadTrue(r) => {
                        constants.insert(*r, Value::from(true));
                    }
                    Instruction::LoadFalse(r) => {
                        constants.insert(*r, Value::from(false));
                    }
                    Instruction::LoadNull(r) => {
                        constants.insert(*r, Value::from(VTag::Null));
                    }
                    Instruction::LoadUndefined(r) => {
                        constants.insert(*r, Value::from(VTag::Undefined));
                    }
                    Instruction::ConditionalBranch(r, if_true, if_false) => {
                        if let Some(c) = constants.get(r) {
                            if c.to_boolean() {
                                let new_ins = Instruction::Branch(*if_true);
                                *ins = new_ins;
                            } else {
                                let new_ins = Instruction::Branch(*if_false);
                                *ins = new_ins;
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }
    }
}

fn ins_for_const(v: Value, r: u16) -> Instruction {
    if v.is_int32() {
        Instruction::LoadInt(r, v.as_int32())
    } else if v.is_number() {
        Instruction::LoadNumber(r, v.to_number().to_bits())
    } else if v.is_bool() {
        if v.to_boolean() {
            Instruction::LoadTrue(r)
        } else {
            Instruction::LoadFalse(r)
        }
    } else if v.is_null() {
        Instruction::LoadNull(r)
    } else if v.is_undefined() {
        Instruction::LoadUndefined(r)
    } else {
        unreachable!()
    }
}
