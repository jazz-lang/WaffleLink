use super::runtime::async_rt::executor::Executor;
use crate::gc::{self, *};
use gc::object::*;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
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
    pub(crate) executor: Executor,
}

impl Isolate {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            heap: Box::into_raw(Box::new(Heap::lazysweep())),
            executor: Executor::new(),
        })
    }

    pub fn executor(&self) -> &Executor {
        &self.executor
    }
    pub fn spawn(&self, task: impl std::future::Future<Output = ()> + 'static + Send) {
        self.executor().spawn(task);
    }

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

    pub fn run(self: &Arc<Self>, entrypoint: impl FnOnce(&Arc<Isolate>) + Send + 'static) {
        let this = self.clone();
        self.executor().block_on(async move {
            let t = this.clone();
            this.spawn(async move {
                entrypoint(&t);
                t.executor().terminate();
            });
            this.executor.run();
            
        });
    }
}

unsafe impl Send for Isolate {}
unsafe impl Sync for Isolate {}
