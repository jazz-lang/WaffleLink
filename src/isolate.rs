use crate::gc::*;
use crate::gc::*;
use std::collections::VecDeque;
/// An isolated WaffleLink execution context.
///
/// All WaffleLink code runs in an isolate, and code can access classes and values only from the same isolate. Different isolates can communicate by sending values through ports.
pub struct Isolate {
    heap: *mut Heap,
}

impl Isolate {
    /// Get Isolate heap
    pub fn heap(&self) -> &mut Heap {
        unsafe { &mut *self.heap }
    }
}
