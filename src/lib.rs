#![allow(dead_code)]
use std::sync::atomic::AtomicU8;
#[macro_export]
macro_rules! log {
    ($($arg: tt)*) => {
        if crate::get_vm().log {
            let lock = std::io::stdout();
            let lock = lock.lock();
            print!("LOG: ");
            println!($($arg)*);
            drop(lock);
        }
    };
}
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
pub mod heap;
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
    pub call_stack: Vec<interpreter::callframe::CallFrame>,
    pub exception: value::Value,
    pub empty_string: value::Value,
    pub stop_world: bool,
    pub opt_jit: bool,
    pub log: bool,
    pub heap: heap::Heap,
}

impl VM {
    pub fn new(stack_start: *const bool) -> Self {
        Self {
            top_call_frame: std::ptr::null_mut(),
            exception: value::Value::undefined(),
            call_stack: vec![],
            stop_world: false,
            log: true,
            #[cfg(feature = "opt-jit")]
            opt_jit: true,
            #[cfg(not(feature = "opt-jit"))]
            opt_jit: false,
            empty_string: value::Value::undefined(),
            heap: heap::Heap::new(stack_start),
        }
    }
    pub fn top_call_frame(&self) -> Option<&mut interpreter::callframe::CallFrame> {
        if self.top_call_frame.is_null() {
            return None;
        } else {
            return Some(unsafe { &mut *self.top_call_frame });
        }
    }

    pub fn exception_addr(&self) -> *const value::Value {
        &self.exception
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

#[repr(C)]
pub struct WaffleResult {
    pub(crate) a: u64,
    pub(crate) b: u64,
}
impl WaffleResult {
    pub fn is_error(&self) -> bool {
        self.a == 1
    }

    pub fn is_okay(&self) -> bool {
        self.a == 0
    }

    pub fn value(&self) -> value::Value {
        unsafe { std::mem::transmute(self.b) }
    }

    pub fn okay(v: value::Value) -> Self {
        Self {
            a: 0,
            b: unsafe { std::mem::transmute(v) },
        }
    }
    pub fn error(v: value::Value) -> Self {
        Self {
            a: 1,
            b: unsafe { std::mem::transmute(v) },
        }
    }
}
pub type WaffleInternalFn = extern "C" fn(&mut interpreter::callframe::CallFrame) -> WaffleResult;
