use parking_lot::Condvar;
use parking_lot::Mutex;

use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
thread_local! {
    pub static THREAD: RefCell<Arc<Thread>> = RefCell::new(Thread::main());
}

pub struct Threads {
    pub threads: Mutex<Vec<Arc<Thread>>>,
    pub cond_join: Condvar,

    pub next_id: AtomicUsize,
    pub safepoint: Mutex<(usize, usize)>,

    pub barrier: Barrier,
}
impl Threads {
    pub fn new() -> Threads {
        Threads {
            threads: Mutex::new(Vec::new()),
            cond_join: Condvar::new(),
            next_id: AtomicUsize::new(1),
            safepoint: Mutex::new((0, 1)),
            barrier: Barrier::new(),
        }
    }

    pub fn attach_current_thread(&self, top: *const u8) {
        assert!(!top.is_null());
        THREAD.with(|thread| {
            let mut threads = self.threads.lock();
            thread
                .borrow()
                .stack_top
                .store(top as usize, Ordering::Relaxed);
            threads.push(thread.borrow().clone());
        });
    }

    pub fn attach_thread(&self, thread: Arc<Thread>) {
        let mut threads = self.threads.lock();
        threads.push(thread);
    }

    pub fn next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn safepoint_id(&self) -> usize {
        let safepoint = self.safepoint.lock();
        safepoint.0
    }

    pub fn safepoint_requested(&self) -> bool {
        let safepoint = self.safepoint.lock();
        safepoint.0 != 0
    }

    pub fn request_safepoint(&self) -> usize {
        let mut safepoint = self.safepoint.lock();
        assert_eq!(safepoint.0, 0);
        safepoint.0 = safepoint.1;
        safepoint.1 += 1;

        safepoint.0
    }

    pub fn clear_safepoint_request(&self) {
        let mut safepoint = self.safepoint.lock();
        assert_ne!(safepoint.0, 0);
        safepoint.0 = 0;
    }

    pub fn detach_current_thread(&self) {
        THREAD.with(|thread| {
            thread.borrow().park();
            let mut threads = self.threads.lock();
            threads.retain(|elem| !Arc::ptr_eq(elem, &*thread.borrow()));
            self.cond_join.notify_all();
        });
    }

    pub fn join_all(&self) {
        let mut threads = self.threads.lock();

        while threads.len() > 0 {
            self.cond_join.wait(&mut threads);
        }
    }

    pub fn each<F>(&self, mut f: F)
    where
        F: FnMut(&Arc<Thread>),
    {
        let threads = self.threads.lock();

        for thread in threads.iter() {
            f(thread)
        }
    }
}
use std::mem::MaybeUninit;
pub struct Thread {
    pub id: usize,
    pub stack_cur: AtomicUsize,
    pub stack_top: AtomicUsize,
    //pub regs: MaybeUninit<jmp_buf>,
    pub state: StateManager,
}
static TID: AtomicUsize = AtomicUsize::new(0);

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}
impl Thread {
    pub fn new() -> Arc<Thread> {
        Thread::with_id(TID.fetch_add(1, Ordering::AcqRel))
    }

    pub fn main() -> Arc<Thread> {
        Thread::with_id(0)
    }

    fn with_id(id: usize) -> Arc<Thread> {
        Arc::new(Thread {
            id,
            //regs: MaybeUninit::uninit(),
            stack_cur: AtomicUsize::new(0),
            stack_top: AtomicUsize::new(0),
            state: StateManager::new(),
        })
    }
    pub fn state(&self) -> ThreadState {
        self.state.state()
    }

    pub fn park(&self) {
        self.state.park();
    }

    pub fn unpark(&self) {
        if super::VM.state.threads.safepoint_id() != 0 {
            block(self, &false);
        }

        self.state.unpark();
    }

    pub fn block(&self, safepoint_id: usize) {
        self.state.block(safepoint_id);
    }

    pub fn unblock(&self) {
        self.state.unblock();
    }

    pub fn in_safepoint(&self, safepoint_id: usize) -> bool {
        self.state.in_safepoint(safepoint_id)
    }
}
pub struct Barrier {
    active: Mutex<usize>,
    done: Condvar,
}

impl Barrier {
    pub fn new() -> Barrier {
        Barrier {
            active: Mutex::new(0),
            done: Condvar::new(),
        }
    }

    pub fn guard(&self, safepoint_id: usize) {
        let mut active = self.active.lock();
        assert_eq!(*active, 0);
        assert_ne!(safepoint_id, 0);
        *active = safepoint_id;
    }

    pub fn resume(&self, safepoint_id: usize) {
        let mut active = self.active.lock();
        assert_eq!(*active, safepoint_id);
        assert_ne!(safepoint_id, 0);
        *active = 0;
        self.done.notify_all();
    }

    pub fn wait(&self, safepoint_id: usize) {
        let mut active = self.active.lock();
        assert_ne!(safepoint_id, 0);

        while *active == safepoint_id {
            self.done.wait(&mut active);
        }
    }
}

pub struct StateManager {
    mtx: Mutex<(ThreadState, usize)>,
}

impl StateManager {
    fn new() -> StateManager {
        StateManager {
            mtx: Mutex::new((ThreadState::Running, 0)),
        }
    }

    fn state(&self) -> ThreadState {
        let mtx = self.mtx.lock();
        mtx.0
    }

    fn park(&self) {
        let mut mtx = self.mtx.lock();
        assert!(mtx.0.is_running());
        mtx.0 = ThreadState::Parked;
    }

    fn unpark(&self) {
        let mut mtx = self.mtx.lock();
        assert!(mtx.0.is_parked());
        mtx.0 = ThreadState::Running;
    }

    fn block(&self, safepoint_id: usize) {
        let mut mtx = self.mtx.lock();
        assert!(mtx.0.is_running());
        mtx.0 = ThreadState::Blocked;
        mtx.1 = safepoint_id;
    }

    fn unblock(&self) {
        let mut mtx = self.mtx.lock();
        assert!(mtx.0.is_blocked());
        mtx.0 = ThreadState::Running;
        mtx.1 = 0;
    }

    fn in_safepoint(&self, safepoint_id: usize) -> bool {
        assert_ne!(safepoint_id, 0);
        let mtx = self.mtx.lock();

        match mtx.0 {
            ThreadState::Running => false,
            ThreadState::Blocked => mtx.1 == safepoint_id,
            ThreadState::Parked => true,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ThreadState {
    Running = 0,
    Parked = 1,
    Blocked = 2,
}

impl From<usize> for ThreadState {
    fn from(value: usize) -> ThreadState {
        match value {
            0 => ThreadState::Running,
            1 => ThreadState::Parked,
            2 => ThreadState::Blocked,
            _ => unreachable!(),
        }
    }
}

impl ThreadState {
    pub fn is_running(&self) -> bool {
        match *self {
            ThreadState::Running => true,
            _ => false,
        }
    }

    pub fn is_parked(&self) -> bool {
        match *self {
            ThreadState::Parked => true,
            _ => false,
        }
    }

    pub fn is_blocked(&self) -> bool {
        match *self {
            ThreadState::Blocked => true,
            _ => false,
        }
    }

    pub fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl Default for ThreadState {
    fn default() -> ThreadState {
        ThreadState::Running
    }
}

#[allow(dead_code)]
#[inline(never)]
fn save_ctx<T>(prev: *const T) {
    THREAD.with(|thread| {
        let thread = thread.borrow();

        // Save registers
        //setjmp(thread.regs.as_ptr() as *mut jmp_buf);
        thread
            .stack_cur
            .store((&prev as *const *const T) as usize, Ordering::Relaxed);
    });
}
pub fn stop_the_world<F, R>(f: F, _prev: *const bool) -> R
where
    F: FnOnce(&[Arc<Thread>]) -> R,
{
    //save_ctx(&prev);
    THREAD.with(|thread| {
        let thread = thread.borrow();
        thread.park();
    });

    let threads = super::VM.state.threads.threads.lock();
    if threads.len() == 1 {
        let ret = f(&*threads);
        THREAD.with(|thread| thread.borrow().unpark());
        return ret;
    }

    let safepoint_id = stop_threads(&*threads);
    let ret = f(&*threads);
    resume_threads(&*threads, safepoint_id);
    THREAD.with(|thread| thread.borrow().unpark());
    ret
}

fn stop_threads(threads: &[Arc<Thread>]) -> usize {
    let thread_self = THREAD.with(|thread| thread.borrow().clone());
    let safepoint_id = super::VM.state.threads.request_safepoint();

    super::VM.state.threads.barrier.guard(safepoint_id);
    // TODO: Arm stack guard or patch code.
    while !all_threads_blocked(&thread_self, threads, safepoint_id) {
        std::sync::atomic::spin_loop_hint();
    }

    safepoint_id
}

fn all_threads_blocked(
    thread_self: &Arc<Thread>,
    threads: &[Arc<Thread>],
    safepoint_id: usize,
) -> bool {
    let mut all_blocked = true;

    for thread in threads {
        if Arc::ptr_eq(thread, thread_self) {
            assert!(thread.state().is_parked());
            continue;
        }

        if !thread.in_safepoint(safepoint_id) {
            all_blocked = false;
        }
    }

    all_blocked
}

fn resume_threads(_threads: &[Arc<Thread>], safepoint_id: usize) {
    super::VM.state.threads.barrier.resume(safepoint_id);
    super::VM.state.threads.clear_safepoint_request();
}

pub extern "C" fn guard_check() {
    let thread = THREAD.with(|thread| thread.borrow().clone());
    let stack_overflow = false; // TODO: Check for stack overflow

    if stack_overflow {
        panic!("Stack overflow"); // this should unwind.
    } else {
        let x = false;
        block(&thread, &x);
    }
}

#[inline(never)]
pub fn block(thread: &Thread, _prev: *const bool) {
    // Save stack pointer
    /*thread
        .stack_cur
        .store((&prev as *const *const bool) as usize, Ordering::Relaxed);
    unsafe {
        // Save registers
        setjmp(thread.regs.as_ptr() as *mut jmp_buf);
    }*/
    //save_ctx(&prev);
    let safepoint_id = super::VM.state.threads.safepoint_id();
    assert_ne!(safepoint_id, 0);
    let state = thread.state();

    match state {
        ThreadState::Running | ThreadState::Parked => {
            thread.block(safepoint_id);
        }
        ThreadState::Blocked => {
            panic!("illegal thread state: thread #{} {:?}", thread.id, state);
        }
    };

    let _mtx = super::VM.state.threads.barrier.wait(safepoint_id);
    thread.unblock();
}
