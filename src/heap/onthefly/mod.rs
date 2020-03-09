/*
*   Copyright (c) 2020 Adel Prokurov
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/

pub mod marking;
pub mod sweeper;
use super::*;
use crate::runtime;
use crate::util::arc::Arc;
use crate::util::mem::Address;
use crossbeam::channel::Receiver;
use crossbeam::deque::Injector;
use crossbeam::deque::Worker;
use freelist::*;
use freelist_alloc::*;
use parking_lot::Mutex;
use runtime::cell::*;
use runtime::process::*;
use space::*;
use std::sync::atomic::{AtomicU8, Ordering};
lazy_static::lazy_static!(
    pub static ref GC: Mutex<OnTheFlyCollector> = Mutex::new(OnTheFlyCollector {
        mark_pool: marking::MarkingPool::new(num_cpus::get()),
        sweeper: sweeper::SweepPool::new(num_cpus::get())
    });
);

pub struct OnTheFlyCollector {
    pub mark_pool: marking::MarkingPool,
    pub sweeper: sweeper::SweepPool,
}

impl OnTheFlyCollector {}
pub const FINISH: u8 = 5;
pub const SWEEPING: u8 = 4;
pub const INIT_SWEEP: u8 = 3;
pub const MARKING: u8 = 2;
pub const SATB: u8 = 1;
pub const NONE: u8 = 0;

pub struct OnTheFlyHeap {
    freelist: FreeListAllocator,
    state: Arc<AtomicU8>,
    injector: Option<Arc<Injector<CellPointer>>>,
    rootset: Vec<CellPointer>,
    sweep_recv: Option<Vec<Receiver<Vec<(Address, usize)>>>>,
    sweeping: Arc<AtomicU8>,
    white: u8,
    needs_gc: bool,
}

impl OnTheFlyHeap {
    pub fn new(heap: usize) -> Self {
        Self {
            freelist: FreeListAllocator::new(Space::new(align_usize(
                heap,
                crate::util::mem::page_size(),
            ))),
            state: Arc::new(AtomicU8::new(0)),
            injector: None,
            rootset: vec![],
            white: CELL_WHITE_A,
            sweep_recv: None,
            sweeping: Arc::new(AtomicU8::new(0)),
            needs_gc: false,
        }
    }
    pub fn trace_proc(&mut self, proc: &Arc<Process>) {
        let channel = proc.local_data().channel.lock();
        channel.trace(|pointer| unsafe {
            self.rootset.push(*pointer);
        });
        proc.trace(|pointer| unsafe {
            self.rootset.push(*pointer);
        });
    }

    fn flip_white(&mut self) {
        if self.white == CELL_WHITE_A {
            self.white = CELL_WHITE_B
        } else {
            self.white = CELL_WHITE_A
        }
    }
    fn live_white(&self) -> u8 {
        if self.white == CELL_WHITE_A {
            CELL_WHITE_B
        } else {
            CELL_WHITE_A
        }
    }
    pub fn collect(&mut self, proc: &Arc<Process>) {
        let state = self.state.load(Ordering::Acquire);
        match state {
            NONE => {
                log::debug!("Initializing concurrent marking...");
                self.state.store(MARKING, Ordering::Relaxed); // no threads try to access 'state' so use relaxed order.
                self.rootset.clear();
                self.flip_white();
                self.trace_proc(proc);
                let gc: parking_lot::MutexGuard<'_, OnTheFlyCollector> = GC.lock();
                let injector = Arc::new(Injector::new());
                while let Some(val) = self.rootset.pop() {
                    injector.push(val);
                }
                self.injector = Some(injector.clone());
                let worker = Worker::new_fifo();
                let (snd, rcv) = crossbeam::channel::unbounded();
                gc.mark_pool.schedule(marking::MarkingJob {
                    process: proc.clone(),
                    queue: worker,
                    injector: injector,
                    state: self.state.clone(),
                    remembered_permanent: Default::default(),
                    snd: snd,
                });
                self.needs_gc = false;
                log::debug!("Concurrent marking is initialized");
                return;
            }
            INIT_SWEEP => {
                log::debug!("Initializing concurrent sweep...");
                let mut recvs = vec![];
                let gc: parking_lot::MutexGuard<'_, OnTheFlyCollector> = GC.lock();
                let mut jobs_count = 0;
                for page in self.freelist.space.pages.iter() {
                    jobs_count += 1;
                    let (job, recv) = sweeper::SweeperJob::new(
                        crate::util::ptr::DerefPointer::new(page),
                        self.white,
                        self.live_white(),
                        self.state.clone(),
                        self.sweeping.clone(),
                    );
                    recvs.push(recv);
                    gc.sweeper.schedule(job);
                }
                self.sweep_recv = Some(recvs);
                self.state.store(SWEEPING, Ordering::Relaxed);
                log::debug!("Concurrent sweep initialized");
            }
            FINISH => {
                log::debug!("Concurrent collection finished");
                assert!(self.sweep_recv.is_some());
                for recv in self.sweep_recv.take().unwrap() {
                    let recv = recv.try_recv().unwrap();
                    for (addr, size) in recv {
                        self.freelist.freelist.add(addr, size);
                    }
                }
                self.state.store(NONE, Ordering::Relaxed);
            }
            SWEEPING => {}
            MARKING => {}

            _ => unimplemented!(),
        }
    }

    /// Wait for "safe" moment to terminate GC.
    pub fn terminate(&mut self) {
        let time = std::time::Instant::now();
        loop {
            let state = self.state.load(Ordering::Acquire);

            if state == FINISH || state == INIT_SWEEP || state == NONE {
                break;
            }
            if time.elapsed().as_millis() >= 2000 {
                // if waiting more than two seconds just break
                break;
            }
            //println!("{}", state);
            std::thread::yield_now();
        }

        self.freelist.space.clear();
    }
}

impl HeapTrait for OnTheFlyHeap {
    fn allocate(&mut self, _: &Arc<Process>, _: GCType, cell: Cell) -> CellPointer {
        let mut needs_gc = false;
        let ptr = self
            .freelist
            .allocate(std::mem::size_of::<Cell>(), &mut needs_gc)
            .to_mut_ptr::<Cell>();
        let state = self.state.load(Ordering::Acquire);
        self.needs_gc = needs_gc && (state == NONE || state == FINISH);
        unsafe {
            ptr.write(cell);
        }

        let ptr = CellPointer {
            raw: crate::util::tagged::TaggedPointer::new(ptr),
        };
        ptr.get_mut()
            .forward
            .atomic_store(self.live_white() as *mut u8);
        ptr
    }

    fn trace_process(&mut self, proc: &Arc<crate::runtime::process::Process>) {
        self.trace_proc(proc);
    }
    fn collect_garbage(
        &mut self,
        proc: &Arc<crate::runtime::process::Process>,
    ) -> Result<(), bool> {
        self.collect(proc);
        Ok(())
    }

    fn should_collect(&self) -> bool {
        let state = self.state.load(Ordering::Acquire);
        self.needs_gc || state == INIT_SWEEP || state == FINISH
    }

    fn enable(&mut self) {}
    fn disable(&mut self) {}
    fn is_enabled(&self) -> bool {
        true
    }

    fn field_write_barrier(&mut self, parent: CellPointer, child: Value) {
        let state = self.state.load(Ordering::Acquire);
        if state != MARKING || !child.is_cell() {
            if child.is_cell() {
                child
                    .as_cell()
                    .get()
                    .forward
                    .atomic_store(self.live_white() as *mut u8);
            }
            return;
        }
        let child = child.as_cell();
        if child.is_permanent() {
            return;
        }
        if parent.get().forward.atomic_load() as u8 != CELL_BLACK && !parent.is_permanent() {
            return;
        }
        if child.get().forward.atomic_load() as u8 != self.white {
            return;
        }

        child.get().forward.atomic_store(CELL_GREY as *mut u8);
        self.injector.as_mut().unwrap().push(child);
    }

    fn write_barrier(&mut self, x: CellPointer) {
        let state = self.state.load(Ordering::Acquire);
        if state != MARKING {
            x.get().forward.atomic_store(self.live_white() as *mut u8);
        } else {
            if x.get().forward.atomic_load() as u8 != CELL_BLACK {
                x.get().forward.atomic_store(CELL_GREY as *mut u8);
                self.injector.as_mut().unwrap().push(x);
            }
        }
    }

    fn clear(&mut self) {
        self.terminate();
    }
}
