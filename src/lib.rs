#![allow(dead_code)]
use std::sync::atomic::AtomicU8;
#[macro_export]
macro_rules! log {
    ($($arg: tt)*) => {
        if crate::LOG.load(std::sync::atomic::Ordering::Relaxed) {
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
pub mod frontend;
pub mod function;
pub mod gc;
pub mod heap;
pub mod interpreter;
pub mod jit;
pub mod mir;
pub mod object;
pub mod pure_nan;
pub mod table;
pub mod utils;
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
    pub constructor: value::Value,
    pub length: value::Value,
    pub not_a_func_exc: value::Value,
    pub prototype: value::Value,
    pub stop_world: bool,
    pub disasm: bool,
    pub opt_jit: bool,
    pub template_jit: bool,
    pub jit_threshold: u32,
    pub log: bool,
    pub heap: heap::Heap,
    pub stubs: JITStubs,
    pub globals: std::collections::HashMap<object::Ref<object::WaffleString>, value::Value>,
}

pub struct JITStubs {
    thunks: parking_lot::Mutex<std::collections::HashMap<fn() -> *const u8, *const u8>>,
}

impl JITStubs {
    pub fn new() -> Self {
        Self {
            thunks: parking_lot::Mutex::new(Default::default()),
        }
    }
    pub fn get_stub(&self, f: fn() -> *const u8) -> *const u8 {
        let mut thunks = self.thunks.lock();
        if let Some(x) = thunks.get(&f) {
            return *x;
        } else {
            let addr = f();
            thunks.insert(f, addr);
            addr
        }
    }
}
pub static LOG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
impl VM {
    pub fn new(stack_start: *const bool) -> Self {
        let mut this = Self {
            top_call_frame: std::ptr::null_mut(),
            exception: value::Value::undefined(),
            call_stack: vec![],
            jit_threshold: 500,
            template_jit: true,
            disasm: false,
            stubs: JITStubs::new(),
            stop_world: false,
            log: true,
            #[cfg(feature = "opt-jit")]
            opt_jit: true,
            #[cfg(not(feature = "opt-jit"))]
            opt_jit: false,
            empty_string: value::Value::undefined(),
            heap: heap::Heap::new(stack_start),
            length: value::Value::undefined(),
            constructor: value::Value::undefined(),
            prototype: value::Value::undefined(),
            not_a_func_exc: value::Value::undefined(),
            globals: Default::default(),
        };
        this.length =
            value::Value::from(object::WaffleString::new(&mut this.heap, "length").cast());
        this.constructor =
            value::Value::from(object::WaffleString::new(&mut this.heap, "constructor").cast());
        this.empty_string =
            value::Value::from(object::WaffleString::new(&mut this.heap, "").cast());
        this.prototype =
            value::Value::from(object::WaffleString::new(&mut this.heap, "prototype").cast());
        this.not_a_func_exc = value::Value::from(
            object::WaffleString::new(&mut this.heap, "function value expected").cast(),
        );
        this
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

    pub fn throw_exception_str(&mut self, s: impl AsRef<str>) -> WaffleResult {
        let val = object::WaffleString::new(&mut self.heap, s);
        self.exception = value::Value::from(val.cast());
        WaffleResult::error(self.exception)
    }

    pub fn allocate<T>(&mut self, val: T) -> object::Ref<T> {
        unsafe {
            let mem = self.heap.allocate(std::mem::size_of::<T>());
            mem.to_mut_ptr::<T>().write(val);
            std::mem::transmute(mem)
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

#[repr(C)]
pub struct WaffleResult {
    pub a: u64,
    pub b: u64,
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
