use crate::gc::{self, *};
use gc::object::*;
use lasso::*;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::current;

pub static INTERNER: Lazy<lasso::ThreadedRodeo<lasso::Spur>> =
    Lazy::new(|| lasso::ThreadedRodeo::new());

/// An isolated WaffleLink execution context.
///
/// All WaffleLink code runs in an isolate, and code can access classes and values only from the same isolate.
/// Different isolates can communicate by sending values through ports.
///
///
///
///
///
pub struct Isolate {
    heap: *mut Heap,
    stack_begin: AtomicUsize,
    cur_thread: AtomicU64,
}

pub fn current_thread_id() -> u64 {
    unsafe { std::mem::transmute(current().id()) }
}

impl Isolate {
    pub fn intern_str(&self, s: &str) -> u32 {
        unsafe { INTERNER.get_or_intern(s).into_usize() as _ }
    }

    pub fn unintern(&self, x: u32) -> Option<&str> {
        INTERNER.try_resolve(&Spur::try_from_usize(x as _).unwrap())
    }
    pub fn update_current_thread(&self) {
        self.cur_thread.store(current_thread_id(), Ordering::AcqRel);
    }
    pub fn current_thread(&self) -> u64 {
        self.cur_thread.load(Ordering::Acquire)
    }
    pub fn new(approx_sp: &usize) -> Arc<Self> {
        let mut this = Arc::new(Self {
            heap: Box::into_raw(Box::new(Heap::lazysweep())),
            stack_begin: AtomicUsize::new(approx_sp as *const usize as usize),
            cur_thread: AtomicU64::new(current_thread_id()),
        });
        this.heap().gc.set_isolate(this.clone());
        this
    }
    pub fn update_stack_begin(&self, to: &usize) -> usize {
        let current = self.stack_begin.load(Ordering::Relaxed);
        self.stack_begin
            .compare_and_swap(current, to as *const usize as _, Ordering::AcqRel)
    }
    /// Get Isolate heap
    pub fn heap(&self) -> &mut Heap {
        unsafe { &mut *self.heap }
    }
    /// Allocate `val` on GC heap and create new `Local` instance in last local scope or in
    /// persistent scope.
    pub fn new_local<T: GcObject>(&self, val: T) -> Local<T> {
        self.heap()
            .gc
            .last_local_scope()
            .unwrap_or(self.heap().gc.persistent_scope())
            .allocate(val)
    }
}

unsafe impl Send for Isolate {}
unsafe impl Sync for Isolate {}

pub type RCIsolate = Arc<Isolate>;
