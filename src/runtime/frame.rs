use super::cell::*;
use super::value::*;
use crate::common::mem::{commit, uncommit, Address};
use crate::common::ptr::*;

#[repr(C)]
pub struct Frame {
    pub rax: Value,
    pub regs: Ptr<Value>,
    pub func: Ptr<Cell>,
    pub this: Value,
    pub arguments: Vec<Value>,
    pub stack: Vec<Value>,
    pub module: Value,

    pub ip: usize,
    pub bp: usize,
    pub try_catch: Vec<u32>,
    pub exit_on_return: bool,
}

impl Frame {
    pub fn new(this: Value, module: Value) -> Self {
        Self {
            this,
            bp: 0,
            ip: 0,
            func: Ptr::null(),
            arguments: vec![],
            try_catch: vec![],
            rax: Value::new_int(0),
            stack: Vec::with_capacity(256),
            regs: Ptr::from_raw(
                commit(
                    crate::common::mem::page_align(std::mem::size_of::<Value>() * 256),
                    false,
                )
                .to_mut_ptr::<u8>(),
            ),
            exit_on_return: false,
            module,
        }
    }

    pub fn native_frame(this: Value, args: Vec<Value>, module: Value) -> Self {
        Self {
            func: Ptr::null(),
            this,
            bp: 0,
            ip: 0,
            rax: Value::new_int(0),
            arguments: args,
            stack: vec![],
            try_catch: vec![],
            regs: Ptr::null(),
            exit_on_return: false,
            module,
        }
    }

    pub fn trace(&self, stack: &mut std::collections::VecDeque<*const Ptr<Cell>>) {
        if self.this.is_cell() {
            stack.push_back(self.this.cell_ref());
        }
        for arg in self.arguments.iter() {
            if arg.is_cell() {
                stack.push_back(arg.cell_ref());
            }
        }

        if !self.regs.is_null() {
            for i in 0..255 {
                let value = self.r(i);
                if value.is_cell() {
                    stack.push_back(value.cell_ref());
                }
            }
        }
        stack.push_back(&self.func);
        if !self.module.is_empty() {
            stack.push_back(self.module.cell_ref());
        }
        stack.push_back(self.module.cell_ref());
    }

    pub fn r(&self, i: usize) -> &mut Value {
        unsafe { &mut *self.regs.offset(i as _).raw }
    }

    pub fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(Value::from(VTag::Undefined))
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        self.stack.clear();
        self.this = Value::empty();
        uncommit(
            Address::from_ptr(self.regs.raw),
            crate::common::mem::page_align(std::mem::size_of::<Value>() * 256),
        );
    }
}
