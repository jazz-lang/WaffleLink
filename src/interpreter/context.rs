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

use crate::bytecode::basicblock::BasicBlock;
use crate::runtime;
use crate::util;
use runtime::cell::*;
use runtime::module::*;
use runtime::process::*;
use runtime::value::*;
use std::vec::Vec;
use util::arc::Arc;
use util::ptr::*;
pub struct Context {
    pub registers: [Value; 48],
    pub stack: Vec<Value>,
    pub module: Arc<Module>,
    pub parent: Option<Ptr<Context>>,
    pub index: usize,
    pub bindex: usize,
    pub code: Arc<Vec<BasicBlock>>,
    pub this: Value,
    pub function: CellPointer,
    pub terminate_upon_return: bool,
    pub return_register: Option<u16>,
    pub n: usize,
    pub in_tail: bool,
}

impl Context {
    pub fn contexts(&self) -> ExecutionContextIterator<'_> {
        ExecutionContextIterator {
            current: Some(self),
        }
    }
    pub fn new() -> Self {
        Self {
            registers: [Value::from(VTag::Undefined); 48],
            stack: Vec::with_capacity(32),
            module: Arc::new(Module {
                globals: vec![],
                name: Arc::new("<>".to_owned()),
                main_fn: Value::empty(),
            }),
            parent: None,
            index: 0,
            this: Value::from(VTag::Undefined),
            bindex: 0,
            return_register: None,
            code: Arc::new(vec![]),
            terminate_upon_return: false,
            function: CellPointer {
                raw: crate::util::tagged::TaggedPointer::null(),
            },
            in_tail: false,
            n: 0,
        }
    }

    pub fn set_register(&mut self, r: u16, value: Value) {
        self.registers[r as usize] = value;
    }

    pub fn get_register(&self, r: u16) -> Value {
        self.registers[r as usize]
    }

    pub fn move_registers(&mut self, r0: u16, r1: u16) {
        let tmp = self.get_register(r0);
        self.registers[r0 as usize] = self.registers[r1 as usize];
        self.registers[r1 as usize] = tmp;
    }

    pub fn trace<F>(&self, mut cb: F)
    where
        F: FnMut(*const CellPointer),
    {
        let mut current = Some(self);
        while let Some(context) = current {
            context.registers.iter().for_each(|x| {
                if x.is_cell() {
                    unsafe { cb(&x.u.ptr) }
                }
            });

            context.stack.iter().for_each(|x| {
                if x.is_cell() {
                    unsafe { cb(&x.u.ptr) }
                }
            });
            context.module.globals.iter().for_each(|x| {
                if x.is_cell() {
                    unsafe { cb(&x.u.ptr) }
                }
            });
            cb(&context.function);
            current = context.parent.as_ref().map(|c| &**c);
        }
    }
}

/// Struct for iterating over an Context and its parent contexts.
pub struct ExecutionContextIterator<'a> {
    current: Option<&'a Context>,
}

impl<'a> Iterator for ExecutionContextIterator<'a> {
    type Item = &'a Context;

    fn next(&mut self) -> Option<&'a Context> {
        if let Some(ctx) = self.current {
            if let Some(parent) = ctx.parent.as_ref() {
                self.current = Some(&*parent);
            } else {
                self.current = None;
            }

            return Some(ctx);
        }

        None
    }
}
