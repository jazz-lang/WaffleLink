//! One byte size lock type. It is used for locking GC allocated objects if necessary.
use std::cell::UnsafeCell;
cfg_if::cfg_if! {
    if #[cfg(feature="multi-threaded")]
     {
        use parking_lot::RawMutex as Lock;
        use parking_lot::lock_api::RawMutex;

        pub struct MLock(UnsafeCell<Option<Lock>> /* we can actually jsut use `RawMutex` but we can catch some errors with using option*/);
        impl MLock {
            pub fn new() -> Self {
                Self(UnsafeCell::new(None))
            }
            #[inline(always)]
            pub fn lock(&self) {
                unsafe {
                    let item = &mut *self.0.get();
                    match item {
                        Some(l) => l.lock(),
                        None => {
                            let lock = Lock::INIT;
                            *item = Some(lock);
                            self.lock();
                        }
                    }
                }
            }
            #[inline(always)]
            pub fn unlock(&self) {
                unsafe {
                    let item = &mut *self.0.get();
                    match item {
                        Some(l) => l.unlock(),
                        None => unreachable!("Trying to unlock uninitialized mutex!")
                    }
                }
            }
            #[inline(always)]
            pub fn try_lock(&self) -> bool {
                unsafe {
                    let item = &mut *self.0.get();
                    match item {
                        Some(l) => l.try_lock(),
                        None => {
                            let lock = Lock::INIT;
                            // lock this mutex
                            lock.lock();
                            *item = Some(lock);
                            true
                        }
                    }
                }
            }
        }
     } else {
        pub struct MLock;

        impl MLock {
            #[inline(always)]
            pub const fn new() -> Self {
                Self
            }
            #[inline(always)]
            pub const fn lock(&self) {}
            #[inline(always)]
            pub const fn unlock(&self) {}
            #[inline(always)]
            pub const fn try_lock(&self) {}
        }
     }
}
