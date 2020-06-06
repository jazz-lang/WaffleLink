use super::gc::*;
use crate::threads::*;
use parking_lot::{lock_api::RawMutex, RawMutex as Mutex};
use std::sync::atomic::Ordering;

/// Wrapper around parking_lot::RawMutex. It is necessary to use this type instead of std or parking lot mutexes
/// to lock your data because this lock will set proper thread state so GC can stop it properly if necessary.
pub struct Lock {
    lock: Mutex,
}

impl Lock {
    pub const INIT: Self = Self::new();
    /// Acquires this mutex, blocking the current thread until it is able to do so.
    pub fn lock(&self) {
        // Until mutex is locked or GC cycle is not finished we yield thread.
        if !self.lock.try_lock() {
            THREAD.with(|x| {
                x.borrow()
                    .state
                    .store(AppThreadState::Parked, Ordering::Release);
            });
            gc_safepoint();
        }
        self.lock.lock();
        // Thread locked! Reset thread state.
        THREAD.with(|x| {
            x.borrow()
                .state
                .store(AppThreadState::Running, Ordering::Release);
        })
    }

    pub fn try_lock(&self) -> bool {
        self.lock.try_lock()
    }

    pub const fn new() -> Self {
        Self { lock: Mutex::INIT }
    }

    pub fn release(&self) {
        self.lock.unlock();
    }
}
