use crate::gc::{self, *};
use gc::object::*;
use std::sync::atomic::AtomicU64;
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
}

impl Isolate {
    /// Get Isolate heap
    pub fn heap(&self) -> &mut Heap {
        unsafe { &mut *self.heap }
    }

    pub fn new_local<T: GcObject>(&self, val: T) -> Local<T> {
        self.heap()
            .gc
            .last_local_scope()
            .unwrap_or(self.heap().gc.persistent_scope())
            .allocate(val)
    }
}
