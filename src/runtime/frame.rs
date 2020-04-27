use super::cell::*;
use super::function::*;
use super::value::*;
use crate::common::mem::{commit, uncommit, Address};
use crate::common::ptr::*;

#[repr(C)]
pub struct Frame {
    pub rax: Value,
    pub regs: Ptr<Value>,
    pub func: Ptr<Cell>,
    pub this: Value,
    pub arguments: Value,
    pub sp: usize,
    pub module: Value,
    pub ip: usize,
    pub bp: usize,
    pub try_catch: Vec<u32>,
    pub exit_on_return: bool,
}

impl Frame {
    pub fn get_code(&self) -> &[BasicBlock] {
        self.func.func_value_unchecked().get_bytecode_unchecked()
    }
    pub fn get_code_mut(&mut self) -> &mut [BasicBlock] {
        self.func
            .func_value_unchecked_mut()
            .get_bytecode_unchecked_mut()
    }
    pub fn new(this: Value, module: Value) -> Self {
        Self {
            this,
            bp: 0,
            ip: 0,
            func: Ptr::null(),
            arguments: Value::empty(),
            try_catch: vec![],
            rax: Value::new_int(0),
            sp: 256,
            regs: Ptr::from_raw(
                commit(
                    crate::common::mem::page_align(std::mem::size_of::<Value>() * 256 + 4 * 1024),
                    false,
                )
                .to_mut_ptr::<u8>(),
            ),
            exit_on_return: false,
            module,
        }
    }

    pub fn native_frame(this: Value, args: Value, module: Value) -> Self {
        Self {
            func: Ptr::null(),
            this,
            bp: 0,
            ip: 0,
            rax: Value::new_int(0),
            arguments: args,
            try_catch: vec![],
            regs: Ptr::from_raw(
                commit(
                    crate::common::mem::page_align(std::mem::size_of::<Value>() * 256 + 4 * 1024),
                    false,
                )
                .to_mut_ptr::<u8>(),
            ),
            sp: 256,

            exit_on_return: false,
            module,
        }
    }

    pub fn trace(&self, stack: &mut std::collections::VecDeque<*const Ptr<Cell>>) {
        if self.this.is_cell() {
            stack.push_back(self.this.cell_ref());
        }
        if self.arguments.is_cell() {
            stack.push_back(self.arguments.cell_ref());
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

    pub fn r(&self, i: u8) -> &mut Value {
        unsafe { &mut *self.regs.offset(i as _).raw }
    }

    pub fn get_constant(&self, ix: u16) -> Value {
        self.func.func_value_unchecked().constants[ix as usize]
    }

    pub fn pop(&mut self) -> Option<&mut Value> {
        if self.sp == 256 {
            return None;
        } else if self.sp == 256 + 512 {
            panic!("Max stack size reached");
        } else {
            unsafe { Some(&mut *self.regs.raw.offset(self.sp as _)) }
        }
    }

    pub fn push(&mut self, val: Value) {
        if self.sp == 256 + 512 {
            panic!("Max stack size reached");
        } else {
            unsafe {
                self.regs.raw.offset(self.sp as _).write(val);
            }
        }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        self.this = Value::empty();
        uncommit(
            Address::from_ptr(self.regs.raw),
            crate::common::mem::page_align(std::mem::size_of::<Value>() * 256 + 8 * 1024),
        );
    }
}
