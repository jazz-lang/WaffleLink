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

use crate::bytecode::*;
use crate::runtime;
use crate::util::arc::Arc;
use basicblock::BasicBlock;
use instruction::{BinOp, Instruction, UnaryOp};
use runtime::cell::*;

pub struct BaselineJIT {
    pub buffer: String,
    vtmp: usize,
}

impl BaselineJIT {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            vtmp: 0,
        }
    }
    pub fn translate_instruction(&mut self, ins: Instruction) {
        match ins {
            Instruction::LoadInt(r, v) => self.write(&format!("r{} = value_int({});", r, v)),
            Instruction::Branch(block) => self.write(&format!("goto bb{};", block)),
            Instruction::ConditionalBranch(value, if_true, if_false) => {
                self.write(&format!("bool cmp_result = value_to_bool(r{});", value));
                self.write(&format!(
                    "if (cmp_result) {{goto bb{};}} else {{goto bb{};}}",
                    if_true, if_false
                ));
            }
            Instruction::BranchIfTrue(value, if_true) => {
                self.write(&format!(
                    "
                        bool cmp_result = value_to_bool(r{});\n
                        if (cmp_result) goto bb{};\n
                    ",
                    value, if_true
                ));
            }
            Instruction::Binary(op, dest, lhs, rhs) => {
                let name = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    BinOp::Mul => "mul",
                    BinOp::Div => "div",
                    BinOp::Mod => "mod",
                    BinOp::Greater => "gt",
                    BinOp::Less => "lt",
                    BinOp::LessOrEqual => "lte",
                    BinOp::GreaterOrEqual => "gte",
                    BinOp::Equal => "eq",
                    BinOp::NotEqual => "neq",
                    _ => unimplemented!(),
                };
                self.write(&format!("r{} = value_{}(r{},r{});", dest, name, lhs, rhs));
            }
            Instruction::Call(dest, callee, argc) => {
                self.write(&format!(
                    "r{} = value_call(proc,worker,stack,r{},{});",
                    dest, callee, argc
                ));
            }
            Instruction::Return(Some(ret)) => {
                self.write(&format!("return_value = r{};", ret));
                self.write(&format!("goto return_from_function;"));
            }
            Instruction::Move(to, from) => {
                self.write(&format!("r{} = r{};", to, from));
            }
            Instruction::Pop(r) => {
                self.write(&format!("r{} = stack_pop(stack)", r));
            }
            Instruction::Push(r) => {
                self.write(&format!("stack_push(stack,r{})", r));
            }
            Instruction::Throw(_) => panic!("Cannot JIT functions thay may throw exceptions"),
            Instruction::CatchBlock { .. } => panic!("Cannot JIT functions with try/catch"),
            _ => unimplemented!(),
        }
    }
    pub fn write(&mut self, string: &str) {
        self.buffer.push_str(string);
    }
    pub fn translate_block(&mut self, bb: &BasicBlock, index: usize) {
        self.write(&format!("bb{}: ", index));
        for ins in bb.instructions.iter() {
            self.translate_instruction(*ins);
        }
    }

    pub fn translate_fn(&mut self, f: &Arc<Function>) {
        self.buffer.clear();
        self.write(&format!(
            "Value baseline_{}(void* worker,void* proc,Stack stack) {{",
            f.name.to_string()
        ));
        for i in 0..32 {
            self.write(&format!("register Value r{};\n", i));
        }
        for (i, bb) in f.code.iter().enumerate() {
            self.translate_block(bb, i);
        }
        self.write("return_from_function: \n");
        self.write("return return_value;");
        self.write("\n}\n");
    }
}
