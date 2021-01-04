use super::block_directory::*;
use super::*;
use atomic::*;
use parking_lot::{lock_api::RawMutex, Condvar, Mutex, MutexGuard};
static TERMINATE: Atomic<bool> = Atomic::new(false);
static SWEEPER_MTX: Mutex<()> = Mutex::const_new(parking_lot::RawMutex::INIT, ());
static SWEEPER_CV: Condvar = Condvar::new();
static FINISH: Atomic<bool> = Atomic::new(false);
pub struct Sweeper {}

impl Sweeper {
    pub fn notify(_lock: MutexGuard<'static, ()>) {
        drop(_lock);
        SWEEPER_CV.notify_one();
    }

    pub fn terminate() -> MutexGuard<'static, ()> {
        TERMINATE.store(true, Ordering::Relaxed);
        SWEEPER_MTX.lock()
    }
}
pub(crate) fn sweeper_run(global: Arc<GlobalAllocator>) {
    std::thread::spawn(move || {
        let mut mtx = SWEEPER_MTX.lock();

        let global = global;
        loop {
            SWEEPER_CV.wait(&mut mtx);
            sweeper_worker(&global);
            if FINISH.load(Ordering::Relaxed) {
                return; // terminate thread.
            }
        }
    });
}
fn sweeper_worker(global: &Arc<GlobalAllocator>) -> bool {
    for dir in global.directories.iter() {
        while let Ok(block) = dir.unswept_list_recv.try_recv() {
            if TERMINATE.load(Ordering::Relaxed) {
                return true;
            }
            // SAFETY: Only *one* thread can access block at the same time
            unsafe {
                (&mut *block).sweep(false);
            }
        }
    }

    true
}
