use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicI8, Ordering};

pub struct TLSState {
    pub safepoint: *mut usize,
    // Whether it is safe to execute GC at the same time.
    pub gc_state: i8,
    //pub alloc: *mut ThreadLocalAllocator,
}
// gc_state = 1 means the thread is doing GC or is waiting for the GC to
//              finish.
pub const GC_STATE_WAITING: i8 = 1;
// gc_state = 2 means the thread is running unmanaged code that can be
//              execute at the same time with the GC.
pub const GC_STATE_SAFE: i8 = 2;

impl TLSState {
    pub fn atomic_gc_state(&self) -> &AtomicI8 {
        as_atomic!(&self.gc_state;AtomicI8)
    }
    #[inline(always)]
    pub fn yieldpoint(&self) {
        unsafe {
            std::ptr::read_volatile(self.safepoint);
        }
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn gc_state_set(&self, state: i8, old_state: i8) -> i8 {
        self.atomic_gc_state().store(state, Ordering::Release);
        if old_state != 0 && state == 0 {
            self.yieldpoint();
        }
        old_state
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn gc_state_save_and_set(&self, state: i8) -> i8 {
        self.gc_state_set(state, self.gc_state)
    }
}

#[thread_local]
static TLS: UnsafeCell<TLSState> = unsafe {
    UnsafeCell::new(TLSState {
        safepoint: crate::safepoint::SAFEPOINT_PAGE.cast(),
        gc_state: 0,
        //alloc: 0 as *mut _,
    })
};

pub fn prepare_thread() {
    get_tls_state().safepoint = unsafe { crate::safepoint::SAFEPOINT_PAGE.cast() };
}

pub fn get_tls_state() -> &'static mut TLSState {
    unsafe { &mut *TLS.get() }
}

pub(crate) fn set_gc_and_wait() {
    let ptls = get_tls_state();
    let state = ptls.gc_state;
    ptls.atomic_gc_state()
        .store(GC_STATE_WAITING, Ordering::Release);
    crate::safepoint::safepoint_wait_gc();
    ptls.atomic_gc_state().store(state, Ordering::Release);
}
