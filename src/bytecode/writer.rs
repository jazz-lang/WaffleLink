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
use super::reader::{TAG_FUN, TAG_STRING};
use crate::runtime;
use runtime::module::Module;
use crate::util::arc::Arc;
use byteorder::{LittleEndian, WriteBytesExt};
use InstructionByte::*;
pub struct BytecodeWriter {
    pub bytecode: Vec<u8>,
}

impl BytecodeWriter {
    pub fn write_u8(&mut self, x: u8) {
        self.bytecode.push(x);
    }
    pub fn write_u16(&mut self, x: u16) {
        self.bytecode.write_u16::<LittleEndian>(x).unwrap();
    }
    pub fn write_u32(&mut self, x: u32) {
        self.bytecode.write_u32::<LittleEndian>(x).unwrap();
    }
    pub fn write_u64(&mut self, x: u64) {
        self.bytecode.write_u64::<LittleEndian>(x).unwrap();
    }

    pub fn write_module(&mut self, m: &Arc<Module>) {
        use hashlink::LinkedHashMap;
        let mut strings = LinkedHashMap::new();
        let mut i = 0;
        //let mut new_global = vec![];
        for global in m.globals.iter() {
            if global.is_cell() {
                if global.as_cell().is_string() {
                    strings.insert(
                        global.to_string(),
                        i,
                    );
                    i += 1;
                } else {
                    /*if let Ok(func) = global.as_cell().function_value() {
                        strings.insert(func.name.to_string(), i);
                        i += 1;
                    }*/
                }
            }
        }

        self.write_u32(strings.len() as _);
        self.write_u32(m.globals.len() as _);
        for (string, _) in strings.iter() {
            self.write_u32(string.len() as _);
            for byte in string.as_bytes() {
                self.write_u8(*byte);
            }
        }

        for (i, global) in m.globals.iter().enumerate() {
            log::trace!("Global {}: {}", i, global.to_string());
            if global.is_cell() {
                if global.as_cell().is_string() {
                    self.write(TAG_STRING);
                    self.write(strings.get(&global.to_string()).copied().unwrap() as u32);
                } else {
                    self.write(TAG_FUN);
                    let cell = global.as_cell();
                    let function_value = cell.function_value().unwrap();
                    self.write(function_value.code.len() as u16);
                    self.write(if function_value.name.to_string() == "main" {
                        1u8
                    } else {
                        0u8
                    });
                    self.write(function_value.argc as i16 as u16);
                    self.write(
                        strings
                            .get(&function_value.name.to_string())
                            .copied()
                            .unwrap() as u32,
                    );
                    for bb in function_value.code.iter() {
                        self.write(bb.instructions.len() as u32);
                        for ins in bb.instructions.iter() {
                            self.write_instruction(*ins);
                        }
                    }
                }
            }
        }
    }

    pub fn write_instruction(&mut self, ins: Instruction) {
        match ins {
            Instruction::LoadNull(r) => {
                self.write_u8(InstructionByte::LOAD_NULL);
                self.write_u8(r as _)
            }
            Instruction::LoadUndefined(r) => {
                self.write(InstructionByte::LOAD_UNDEF);
                self.write(r as u8);
            }
            Instruction::LoadInt(r, i) => {
                self.write(InstructionByte::LOAD_INT);
                self.write(r as u8);
                self.write(i as u32);
            }
            Instruction::LoadNumber(r, u) => {
                self.write(LOAD_NUM);
                self.write(r as u8);
                self.write(u as u64);
            }
            Instruction::LoadTrue(r) => {
                self.write(LOAD_TRUE);
                self.write(r as u8);
            }
            Instruction::LoadFalse(r) => {
                self.write(LOAD_FALSE);
                self.write(r as u8);
            }
            Instruction::LoadById(r0, r1, id) => {
                self.write(LOAD_BY_ID);
                self.write(r0 as u8);
                self.write(r1 as u8);
                self.write(id as u32);
            }
            Instruction::StoreById(r0, r1, id) => {
                self.write(STORE_BY_ID);
                self.write(r0 as u8);
                self.write(r1 as u8);
                self.write(id as u32);
            }
            Instruction::LoadByIndex(r0, r1, id) => {
                self.write(LOAD_BY_INDEX);
                self.write(r0 as u8);
                self.write(r1 as u8);
                self.write(id);
            }
            Instruction::StoreByIndex(r0, r1, id) => {
                self.write(STORE_BY_INDEX);
                self.write(r0 as u8);
                self.write(r1 as u8);
                self.write(id);
            }
            Instruction::LoadByValue(r0, r1, r2) => {
                self.write(LOAD_BY_VALUE);
                self.write(r0 as u8);
                self.write(r1 as u8);
                self.write(r2 as u8);
            }
            Instruction::StoreByValue(r0, r1, r2) => {
                self.write(STORE_BY_VALUE);
                self.write(r0 as u8);
                self.write(r1 as u8);
                self.write(r2 as u8);
            }
            Instruction::LoadStaticById(r0, r1) => {
                self.write(LOAD_STATIC_BY_ID);
                self.write(r0 as u8);
                self.write(r1);
            }
            Instruction::StoreStaticById(r0, r1) => {
                self.write(STORE_STATIC_BY_ID);
                self.write(r0 as u8);
                self.write(r1);
            }
            Instruction::StoreStack(r0, ss0) => {
                self.write(STORE_STACK);
                self.write(r0 as u8);
                self.write(ss0);
            }
            Instruction::LoadStack(r0, ss0) => {
                self.write(LOAD_STACK);
                self.write(r0 as u8);
                self.write(ss0);
            }
            Instruction::ConditionalBranch(r0, if_true, if_false) => {
                self.write(CONDITIONAL_BRANCH);
                self.write(r0 as u8);
                self.write(if_true);
                self.write(if_false);
            }
            Instruction::Branch(to) => {
                self.write(BRANCH);
                self.write(to);
            }
            Instruction::BranchIfTrue(r0, to) => {
                self.write(BRANCH_IF_TRUE);
                self.write(r0 as u8);
                self.write(to);
            }
            Instruction::BranchIfFalse(r0, to) => {
                self.write(BRANCH_IF_FALSE);
                self.write(r0 as u8);
                self.write(to);
            }
            Instruction::CatchBlock(r0, bb) => {
                self.write(CATCH_BLOCK);
                self.write(r0 as u8);
                self.write(bb);
            }
            Instruction::Throw(r) => {
                self.write(THROW);
                self.write(r as u8);
            }
            Instruction::MakeEnv(f, c) => {
                self.write(MAKE_ENV);
                self.write(f as u8);
                self.write(c);
            }
            Instruction::Return(Some(r)) => {
                self.write(RETURN);
                self.write(r as u8);
            }
            Instruction::Push(r) => {
                self.write(PUSH);
                self.write(r as u8);
            }
            Instruction::Pop(r) => {
                self.write(POP);
                self.write(r as u8);
            }
            Instruction::Call(r0, r1, c) => {
                self.write(CALL);
                self.write(r0 as u8);
                self.write(r1 as u8);
                self.write_u16(c);
            }
            Instruction::TailCall(r0, r1, c) => {
                self.write(TAIL_CALL);
                self.write(r0 as u8);
                self.write(r1 as u8);
                self.write_u16(c);
            }
            Instruction::VirtCall(r0, r1, r2, c) => {
                self.write(VIRT_CALL);
                self.write(r0 as u8);
                self.write(r1 as u8);
                self.write(r2 as u8);
                self.write(c);
            }
            Instruction::New(r0, r1, c) => {
                self.write(NEW);
                self.write(r0 as u8);
                self.write(r1 as u8);
                self.write(c);
            }
            Instruction::Gc => self.write(GC),
            Instruction::GcSafepoint => self.write(GC_SAFEPOINT),
            Instruction::Binary(op, dest, lhs, rhs) => {
                let b = match op {
                    BinOp::Add => ADD,
                    BinOp::Sub => SUB,
                    BinOp::Mul => MUL,
                    BinOp::Div => DIV,
                    BinOp::Mod => MOD,
                    BinOp::Rsh => RSH,
                    BinOp::Lsh => LSH,
                    BinOp::Greater => GREATER,
                    BinOp::Less => LESS,
                    BinOp::GreaterOrEqual => GREATER_OR_EQUAL,
                    BinOp::LessOrEqual => LESS_OR_EQUAL,
                    BinOp::And => AND,
                    BinOp::Or => OR,
                    BinOp::Xor => XOR,
                    BinOp::Equal => EQUAL,
                    BinOp::NotEqual => NOT_EQUAL,
                };
                self.write(b);
                self.write(dest as u8);
                self.write(lhs as u8);
                self.write(rhs as u8);
            }
            Instruction::Unary(op, dest, lhs) => {
                match op {
                    UnaryOp::Not => self.write(NOT),
                    UnaryOp::Neg => self.write(NEG),
                };
                self.write(dest as u8);
                self.write(lhs as u8);
            }
            Instruction::Move(to, from) => {
                self.write(MOVE);
                self.write(to as u8);
                self.write(from as u8);
            }
            Instruction::LoadConst(r, c) => {
                self.write(LOAD_CONST);
                self.write(r as u8);
                self.write(c as u16);
            }
            Instruction::LoadThis(r) => {
                self.write(LOAD_THIS);
                self.write(r as u8);
            }
            Instruction::SetThis(r) => {
                self.write(SET_THIS);
                self.write(r as u8);
            }
            Instruction::LoadCurrentModule(r) => {
                self.write(LOAD_CURRENT_MODULE);
                self.write(r as u8);
            }
            Instruction::StoreUpvalue(r, s) => {
                self.write(STORE_UPVALUE);
                self.write(r as u8);
                self.write(s);
            }
            Instruction::LoadUpvalue(r, s) => {
                self.write(LOAD_UPVALUE);
                self.write(r as u8);
                self.write(s);
            }
            _ => unimplemented!(),
        }
    }
}

pub trait Writer<T> {
    fn write(&mut self, _: T);
}

impl Writer<u8> for BytecodeWriter {
    fn write(&mut self, x: u8) {
        self.write_u8(x)
    }
}

impl Writer<u16> for BytecodeWriter {
    fn write(&mut self, x: u16) {
        self.write_u16(x);
    }
}

impl Writer<u32> for BytecodeWriter {
    fn write(&mut self, x: u32) {
        self.write_u32(x);
    }
}

impl Writer<u64> for BytecodeWriter {
    fn write(&mut self, x: u64) {
        self.write_u64(x);
    }
}
