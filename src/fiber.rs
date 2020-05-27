use super::gc::*;
use super::state::*;
use std::sync::atomic::{AtomicBool, Ordering};

/// Fiber is stackfull coroutines.
pub struct Fiber {
    /// If set to bool and multi-threaded build is used then runtime will panic
    /// because it is impossible to resume fiber in two threads at the same time.
    pub running: AtomicBool,
    /// If fiber is terminated then this flag is true.
    pub terminated: AtomicBool,
    pub call_stack: CallStack,
    /// Fiber that was exected before current fiber.
    pub prev: Option<Handle<Fiber>>,
}

impl Collectable for Fiber {
    fn walk_references(&self, trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        if let Some(ref item) = self.prev {
            item.walk_references(trace);
        }
    }
}
