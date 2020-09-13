use super::Address;
use crossbeam_deque::{Injector, Steal, Stealer, Worker};
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

use super::pmarking::{Segment, Terminator};

pub struct ConcMarkingTask {
    task_id: usize,
    local: Segment,
    injector: Arc<Injector<Address>>,
    stealers: Arc<Vec<Stealer<Address>>>,
    worker: Worker<Address>,
    terminator: Arc<Terminator>,
    marked: usize,
}

impl ConcMarkingTask {
    fn pop(&mut self) -> Option<Address> {
        self.pop_local()
            .or_else(|| self.pop_worker())
            .or_else(|| self.pop_global())
            .or_else(|| self.steal())
    }

    fn pop_local(&mut self) -> Option<Address> {
        if self.local.is_empty() {
            return None;
        }

        let obj = self.local.pop().expect("should be non-empty");
        Some(obj)
    }

    fn pop_worker(&mut self) -> Option<Address> {
        self.worker.pop()
    }

    fn pop_global(&mut self) -> Option<Address> {
        loop {
            let result = self.injector.steal_batch_and_pop(&mut self.worker);

            match result {
                Steal::Empty => break,
                Steal::Success(value) => return Some(value),
                Steal::Retry => continue,
            }
        }

        None
    }
    fn steal(&self) -> Option<Address> {
        if self.stealers.len() == 1 {
            return None;
        }

        let mut rng = thread_rng();
        let range = Uniform::new(0, self.stealers.len());

        for _ in 0..2 * self.stealers.len() {
            let mut stealer_id = self.task_id;

            while stealer_id == self.task_id {
                stealer_id = range.sample(&mut rng);
            }

            let stealer = &self.stealers[stealer_id];

            loop {
                match stealer.steal_batch_and_pop(&self.worker) {
                    Steal::Empty => break,
                    Steal::Success(gc) => return Some(gc),
                    Steal::Retry => continue,
                }
            }
        }

        None
    }
}

use parking_lot::{Condvar, Mutex};

pub struct ConcurrentMarking {
    wake: Condvar,
    lock: Mutex<()>,
    injector: Arc<Injector<Address>>,
    pool: ThreadPool,
}
