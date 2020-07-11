#![allow(dead_code)]
use std::sync::atomic::AtomicU8;

macro_rules! offset_of {
    ($ty: ty, $field: ident) => {
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    };
}

#[macro_export]
macro_rules! declare_call_frame {
    ($vm: expr) => {
        unsafe { &mut *vm.top_call_frame }
    };
}
pub(crate) static mut SAFEPOINT_PAGE: AtomicU8 = AtomicU8::new(0);
pub mod bigint;
pub mod builtins;
pub mod bytecode;
pub mod bytecompiler;
pub mod function;
pub mod gc;
pub mod interpreter;
pub mod jit;
pub mod object;
pub mod pure_nan;
pub mod value;
pub mod vtable;
pub struct MutatingVecIter<'a, T>(&'a mut Vec<T>, usize);

impl<'a, T> MutatingVecIter<'a, T> {
    pub fn push(&mut self, item: T) {
        self.0.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}

impl<'a, T> std::iter::Iterator for MutatingVecIter<'a, T> {
    type Item = *mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.1 < self.0.len() {
            self.1 += 1;
            let ix = self.1 - 1;
            return Some(unsafe { self.0.get_unchecked_mut(ix) });
        }
        None
    }
}

pub struct VM {
    pub top_call_frame: *mut interpreter::callframe::CallFrame,
}

impl VM {
    pub fn new() -> Self {
        Self {
            top_call_frame: std::ptr::null_mut(),
        }
    }
    pub fn top_call_frame(&self) -> Option<&mut interpreter::callframe::CallFrame> {
        if self.top_call_frame.is_null() {
            return None;
        } else {
            return Some(unsafe { &mut *self.top_call_frame });
        }
    }
}

pub static mut VM_PTR: *mut VM = std::ptr::null_mut();

pub fn set_vm(vm: *const VM) {
    unsafe {
        VM_PTR = vm as *mut _;
    }
}

pub fn get_vm() -> &'static mut VM {
    unsafe { &mut *VM_PTR }
}
