use parking_lot::Mutex;
use std::sync::atomic::{AtomicI8, Ordering};
use std::{cell::UnsafeCell, ptr::null_mut};
pub struct TLSState {
    pub safepoint: *mut usize,
    // Whether it is safe to execute GC at the same time.
    pub gc_state: i8,
    pub stack_start: *mut u8,
    pub stack_end: *mut u8,
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

static OK_LOAD: usize = 0;

#[thread_local]
static TLS: UnsafeCell<TLSState> = {
    UnsafeCell::new(TLSState {
        // if some thread puts yieldpoint but is not registered in GC we do not want to access SAFEPOINT_PAGE here.
        safepoint: &OK_LOAD as *const usize as _,
        gc_state: 0,
        stack_end: null_mut(),
        stack_start: null_mut(), //alloc: 0 as *mut _,
    })
};

#[inline(never)]
pub fn prepare_thread(stack: *const bool) {
    get_tls_state().safepoint = unsafe { crate::safepoint::SAFEPOINT_PAGE.cast() };
    get_tls_state().stack_start = stack as *mut u8;
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

pub struct Threads {
    pub threads: Mutex<Vec<*mut TLSState>>,
}

impl Threads {
    pub fn new() -> Self {
        Self {
            threads: Mutex::new(Vec::with_capacity(2)),
        }
    }
}

pub static THREADS: once_cell::sync::Lazy<Threads> = once_cell::sync::Lazy::new(|| Threads::new());

pub fn spawn_rt_thread<F, R>(f: F) -> std::thread::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    std::thread::spawn(|| {
        let begin = false;
        let threads = &*THREADS;
        let mut lock = threads.threads.lock();
        prepare_thread(&begin);
        let tls = get_tls_state() as *mut _;
        lock.push(get_tls_state() as *mut _);
        drop(lock);
        let result = f();
        let mut lock = threads.threads.lock();
        lock.retain(|x| *x != tls);
        result
    })
}

static HAS_MAIN: AtomicI8 = AtomicI8::new(0);

pub fn register_main_thread(begin: *const bool) {
    assert!(
        HAS_MAIN.compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed) == Ok(1),
        "main thread already registered"
    );

    let mut lock = THREADS.threads.lock();
    prepare_thread(begin);
    //let tls = get_tls_state() as *mut _;
    lock.push(get_tls_state() as *mut _);
}

unsafe impl Send for Threads {}
unsafe impl Sync for Threads {}
