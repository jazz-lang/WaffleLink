use super::Address;
use crossbeam_deque::{Injector, Steal, Stealer, Worker};
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use scoped_threadpool::Pool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

/// Executes marking in NUM_CPUS threads
pub fn start(rootset: &[Address], threadpool: &mut Pool, gc_ty: u8) -> flume::Receiver<Address> {
    let number_workers = threadpool.thread_count() as usize;
    let mut workers = Vec::with_capacity(number_workers);
    let mut stealers = Vec::with_capacity(number_workers);
    let injector = Injector::new();

    for _ in 0..number_workers {
        let w = Worker::new_lifo();
        let s = w.stealer();
        workers.push(w);
        stealers.push(s);
    }

    for root in rootset {
        injector.push(*root);
    }

    let terminator = Terminator::new(number_workers);
    let (snd, recv) = flume::unbounded();
    threadpool.scoped(|scoped| {
        for (task_id, worker) in workers.into_iter().enumerate() {
            let injector = &injector;
            let stealers = &stealers;
            let terminator = &terminator;
            let snd = snd.clone();
            scoped.execute(move || {
                let mut task = MarkingTask {
                    task_id,
                    local: Segment::new(),
                    worker,
                    injector,
                    stealers,
                    terminator,
                    blacklist: snd,
                    marked: 0,
                    gc_ty,
                };

                task.run();
            });
        }
    });
    recv
}
unsafe impl Send for MarkingTask<'_> {}
unsafe impl Send for Segment {}

pub struct Terminator {
    const_nworkers: usize,
    nworkers: AtomicUsize,
}

impl Terminator {
    pub fn new(number_workers: usize) -> Terminator {
        Terminator {
            const_nworkers: number_workers,
            nworkers: AtomicUsize::new(number_workers),
        }
    }

    pub fn try_terminate(&self) -> bool {
        if self.const_nworkers == 1 {
            return true;
        }

        if self.decrease_workers() {
            // reached 0, no need to wait
            return true;
        }

        thread::sleep(Duration::from_micros(1));
        self.zero_or_increase_workers()
    }

    fn decrease_workers(&self) -> bool {
        self.nworkers.fetch_sub(1, Ordering::Relaxed) == 1
    }

    fn zero_or_increase_workers(&self) -> bool {
        let mut nworkers = self.nworkers.load(Ordering::Relaxed);

        loop {
            if nworkers == 0 {
                return true;
            }

            let prev_nworkers =
                self.nworkers
                    .compare_and_swap(nworkers, nworkers + 1, Ordering::Relaxed);

            if nworkers == prev_nworkers {
                // Value was successfully increased again, workers didn't terminate in time. There is still work left.
                return false;
            }

            nworkers = prev_nworkers;
        }
    }
}

struct MarkingTask<'a> {
    task_id: usize,
    local: Segment,
    worker: Worker<Address>,
    injector: &'a Injector<Address>,
    stealers: &'a [Stealer<Address>],
    terminator: &'a Terminator,
    marked: usize,
    blacklist: flume::Sender<Address>,
    gc_ty: u8,
}

impl<'a> MarkingTask<'a> {
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

    fn run(&mut self) {
        unsafe {
            loop {
                let object_addr = if let Some(addr) = self.pop() {
                    addr
                } else if self.terminator.try_terminate() {
                    break;
                } else {
                    continue;
                };

                let object = object_addr.to_mut_obj();
                if object.header.next_tag() != self.gc_ty {
                    if object.header.to_blue() {
                        object.trait_object().visit_references(&mut |item| {
                            let object = item as *mut super::GcBox<()>;
                            let obj = &mut *object;
                            if obj.header.white_to_gray() {
                                self.push(Address::from_ptr(item));
                            }
                        });
                        match self.blacklist.send(object_addr) {
                            Ok(_) => (),
                            Err(_) => std::hint::unreachable_unchecked(), // no way channel is full (we use unbounded) or dropped (scoped threadpool)
                        }
                    }
                } else if object.header.gray_to_black() {
                    if super::GC_VERBOSE_LOG {
                        eprintln!("---thread #{}: mark {:p}", self.task_id, object);
                    }

                    debug_assert!(object.header.tag() == super::GC_BLACK);
                    object.trait_object().visit_references(&mut |item| {
                        let object = item as *mut super::GcBox<()>;
                        let obj = &mut *object;
                        if obj.header.white_to_gray() {
                            self.push(Address::from_ptr(item));
                        }
                    });
                } else {
                    debug_assert!(object.header.tag() == super::GC_BLACK);
                }
            }
        }
    }

    fn push(&mut self, addr: Address) {
        if self.local.has_capacity() {
            self.local.push(addr);
            self.defensive_push();
        } else {
            self.worker.push(addr);
        }
    }

    fn defensive_push(&mut self) {
        self.marked += 1;

        if self.marked > 256 {
            if self.local.len() > 4 {
                let target_len = self.local.len() / 2;

                while self.local.len() > target_len {
                    let val = self.local.pop().unwrap();
                    self.injector.push(val);
                }
            }

            self.marked = 0;
        }
    }
}

const SEGMENT_SIZE: usize = 64;

pub struct Segment {
    data: Vec<Address>,
}

impl Segment {
    pub fn new() -> Segment {
        Segment {
            data: Vec::with_capacity(SEGMENT_SIZE),
        }
    }

    pub fn empty() -> Segment {
        Segment { data: Vec::new() }
    }

    pub fn with(addr: Address) -> Segment {
        let mut segment = Segment::new();
        segment.data.push(addr);

        segment
    }

    pub fn has_capacity(&self) -> bool {
        self.data.len() < SEGMENT_SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn push(&mut self, addr: Address) {
        debug_assert!(self.has_capacity());
        self.data.push(addr);
    }

    pub fn pop(&mut self) -> Option<Address> {
        self.data.pop()
    }

    pub fn len(&mut self) -> usize {
        self.data.len()
    }
}
