use crate::threading::get_tls_state;
use crate::utils::VolatileCell;
use atomic::Atomic;
use std::sync::atomic::Ordering;
use std::thread::{current as thread_self, ThreadId};
pub struct Mutex {
    pub(crate) owner: Atomic<Option<ThreadId>>,
    pub(crate) count: VolatileCell<u32>,
}

impl Mutex {
    unsafe fn wait(&self, safepoint: i32) {
        let this = std::thread::current().id();
        let mut owner = self.owner.load(Ordering::Relaxed);
        if let Some(owner) = owner {
            if owner == this {
                self.count.set(self.count.get() + 1);
                return;
            }
        }

        loop {
            if owner.is_none()
                && self.owner.compare_exchange(
                    None,
                    Some(this),
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                ) == Ok(owner)
            {
                self.count.set(1);
                return;
            }
            if safepoint != 0 {
                // GC might be waiting for us to reach yieldpoint and current thread might be waiting for
                // thread that already is in yieldpoint
                let ptls = get_tls_state();
                ptls.yieldpoint();
            }
            std::sync::atomic::spin_loop_hint();
            owner = self.owner.load(Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn unlock_nogc(&self) {
        assert!(
            self.owner.load(Ordering::Relaxed) == Some(thread_self().id()),
            "Unblocking a lock in different thread"
        );

        self.count.set(self.count.get() - 1);
        if self.count.get() == 0 {
            self.owner.store(None, Ordering::Release);
        }
    }
    #[inline]
    pub fn unlock(&self) {
        self.unlock_nogc();
    }
    #[inline]
    pub fn lock(&self) {
        unsafe {
            self.wait(1);
        }
    }
    #[inline]
    pub fn lock_nogc(&self) {
        unsafe { self.wait(0) }
    }
    #[inline]
    pub fn try_lock_nogc(&self) -> bool {
        let this = thread_self().id();
        let owner = self.owner.load(Ordering::Acquire);
        if owner == Some(this) {
            self.count.set(self.count.get() + 1);
            return true;
        }

        if owner.is_none()
            && self
                .owner
                .compare_exchange(None, Some(this), Ordering::AcqRel, Ordering::Relaxed)
                == Ok(None)
        {
            self.count.set(1);
            return true;
        }
        return false;
    }
    #[inline]
    pub const fn new() -> Self {
        Self {
            count: VolatileCell::new(0),
            owner: Atomic::new(None),
        }
    }
}

unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}
