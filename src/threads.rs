use super::gc::*;
use parking_lot::{Condvar, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
pub static GC_CYCLE: AtomicBool = AtomicBool::new(false);
fn gc_thread_yield(attempt: u64) {
    if attempt >= 2 {
        std::thread::sleep(std::time::Duration::from_micros((attempt - 2) * 1000));
    } else {
        std::thread::yield_now();
    }
}

/// GC safepoint implementation.
///
/// If GC should stop then this function will suspend thread until GC finishes.
pub fn gc_safepoint() {
    let mut attempt = 0;
    if GC_CYCLE.load(Ordering::Acquire) {
        THREAD.with(|item| {
            item.borrow()
                .state
                .store(AppThreadState::InSafepoint, Ordering::Release);
        });
        while GC_CYCLE.load(Ordering::Acquire) {
            gc_thread_yield(attempt);
            attempt += 1;
        }
        // Thread state should be changed to `Running` inside GC.
    }
}
use super::arc::ArcWithoutWeak as Arc;
use std::cell::RefCell;

use atomig::{Atom, Atomic};

thread_local! {
    pub static THREAD: RefCell<Arc<AppThread>> = {
        let id = super::MACHINE.threads.next_id();
        RefCell::new(Arc::new(AppThread::with_id(id)))
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum AppThreadState {
    Running = 0,
    Parked = 1,
    InSafepoint = 2,
    NotStarted = 3,
}

impl Atom for AppThreadState {
    type Repr = u8;

    fn pack(self) -> u8 {
        self as u8
    }

    fn unpack(src: Self::Repr) -> Self {
        match src {
            0 => Self::Running,
            1 => Self::Parked,
            2 => Self::NotStarted,
            _ => unreachable!(),
        }
    }
}

use std::cell::UnsafeCell;

pub struct AppThread {
    pub id: usize,
    pub roots: RefCell<Vec<*mut RootInner<dyn Collectable>>>,
    pub state: Atomic<AppThreadState>,
    pub vm_state: UnsafeCell<super::state::State>,
}

impl AppThread {
    pub fn with_id(id: usize) -> Self {
        Self {
            id,
            roots: RefCell::new(vec![]),
            state: Atomic::new(AppThreadState::NotStarted),
            vm_state: UnsafeCell::new(super::state::State::new()),
        }
    }

    pub fn vm_state(&self) -> &mut super::state::State {
        unsafe { &mut *self.vm_state.get() }
    }
    pub fn roots(&self, mut f: impl FnMut(*const Handle<dyn Collectable>)) {
        self.roots.borrow_mut().retain(|item| unsafe {
            let item = &mut **item;
            if item.refcount.load(Ordering::Relaxed) == 0 {
                false
            } else {
                f(&item.handle);
                true
            }
        });
    }
    pub fn main() -> Self {
        Self::with_id(0)
    }
}

pub struct Threads {
    pub cond_join: Condvar,
    pub next_id: AtomicUsize,
    pub threads: Mutex<Vec<Arc<AppThread>>>,
}

impl Threads {
    pub fn new() -> Self {
        Self {
            cond_join: Condvar::new(),
            next_id: AtomicUsize::new(0),
            threads: Mutex::new(vec![]),
        }
    }
    pub fn next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn attach_current_thread(&self) {
        THREAD.with(|thread| {
            let mut threads = self.threads.lock();
            threads.push(thread.borrow().clone());
        });
    }

    pub fn attach_thread(&self, t: Arc<AppThread>) {
        self.threads.lock().push(t);
    }

    pub fn detach_current_thread(&self) {
        THREAD.with(|thread| {
            let th = &*thread.borrow();
            let mut threads = self.threads.lock();
            threads.retain(|elem| !Arc::ptr_eq(th, elem));
            self.cond_join.notify_all();
        })
    }

    pub fn stop_the_world<R>(&self, mut f: impl FnMut(&[Arc<AppThread>]) -> R) -> R {
        // lock threads from starting or exiting.
        let threads = self.threads.lock();
        GC_CYCLE.store(true, Ordering::Release);
        let thread_self = THREAD.with(|thread| thread.borrow().clone());

        thread_self
            .state
            .store(AppThreadState::InSafepoint, Ordering::Relaxed);
        if threads.len() == 1 {
            let r = f(&*threads);
            thread_self
                .state
                .store(AppThreadState::Running, Ordering::Relaxed);
            GC_CYCLE.store(false, Ordering::Release);
            return r;
        }
        let mut attempt = 0;
        while !self.all_threads_parked(&thread_self, &*threads) {
            gc_thread_yield(attempt);
            attempt += 1;
        }

        let r = f(&*threads);
        GC_CYCLE.store(false, Ordering::Release);
        for thread in threads.iter() {
            thread
                .state
                .store(AppThreadState::Running, Ordering::Relaxed);
        }
        r
    }

    fn all_threads_parked(&self, this: &Arc<AppThread>, threads: &[Arc<AppThread>]) -> bool {
        for thread in threads.iter() {
            if Arc::ptr_eq(this, thread) {
                continue;
            }
            let s = thread.state.load(Ordering::Acquire);
            if s != AppThreadState::InSafepoint && s != AppThreadState::Parked {
                return false;
            }
        }

        true
    }
}
