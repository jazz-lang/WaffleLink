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

use super::*;
use crate::heap;
use crate::runtime;
use crate::runtime::cell::*;
use crate::runtime::scheduler;
use crate::util::arc::Arc;
use crate::util::mem::*;
use crate::util::ptr::DerefPointer;
use crossbeam::channel::{unbounded, Receiver, Sender};
use heap::freelist::*;
use heap::space::Page;
use runtime::process::*;
use runtime::state::*;
use scheduler::join_list::JoinList;
use scheduler::queue::*;
use scheduler::state::*;
use scheduler::worker::Worker as WorkerTrait;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread;
pub struct SweeperJob {
    page: DerefPointer<Page>,
    /// dead objects color
    white1: u8,
    /// these objects will be "dead" in next collection cycle.
    white2: u8,
    /// Thread that launched this job will wait for receiving new freelist
    freelist: Sender<Vec<(Address, usize)>>,
    state: Arc<AtomicU8>,
    sweeping: Arc<AtomicU8>,
}

impl SweeperJob {
    pub fn new(
        page: DerefPointer<Page>,
        white1: u8,
        white2: u8,
        state: Arc<AtomicU8>,
        sweeping: Arc<AtomicU8>,
    ) -> (SweeperJob, Receiver<Vec<(Address, usize)>>) {
        let (sender, receiver) = unbounded();
        let job = Self {
            freelist: sender,
            page,
            white1,
            white2,
            state,
            sweeping,
        };

        (job, receiver)
    }
    fn is_dead(&self, cell: CellPointer) -> bool {
        cell.get().forward.atomic_load() as u8 == self.white1
    }

    pub fn perform(&self, _: &State) {
        while self.state.load(Ordering::Acquire) != SWEEPING {
            std::thread::yield_now();
        }
        let page = &*self.page;
        let mut garbage_start = Address::null();
        let end = page.top;
        log::trace!(
            "Worker '{}': Sweeping memory page from {:p} to {:p} (memory page limit is {:p})",
            std::thread::current().name().unwrap(),
            page.data.to_ptr::<u8>(),
            page.top.to_ptr::<u8>(),
            page.limit.to_ptr::<u8>()
        );
        let mut scan = page.data;
        let mut freelist = Vec::new();
        macro_rules! add_freelist {
            ($start: expr,$end: expr) => {
                if $start.is_non_null() {
                    let size = $end.offset_from($start);
                    freelist.push(($start, size));
                }
            };
        }
        while scan < end {
            let cell_ptr = scan.to_mut_ptr::<Cell>();
            let cell = CellPointer {
                raw: crate::util::tagged::TaggedPointer::new(cell_ptr),
            };

            if self.is_dead(cell) && cell.get().generation != 127 {
                if !garbage_start.is_non_null() {
                    garbage_start = Address::from_ptr(cell_ptr);
                }
                log::trace!(
                    "Worker '{}': Sweep {:p} '{}'",
                    std::thread::current().name().unwrap(),
                    cell_ptr,
                    cell
                );
                unsafe {
                    std::ptr::drop_in_place(cell_ptr);
                }
                cell.get_mut().generation = 127;
            } else {
                cell.get().forward.atomic_store(self.white2 as *mut u8);
                add_freelist!(garbage_start, Address::from_ptr(cell_ptr));
                garbage_start = Address::null();
            }

            scan = scan.offset(std::mem::size_of::<Cell>());
        }
        add_freelist!(garbage_start, page.limit);
        let _ = self.freelist.send(freelist);
        let count = self.sweeping.fetch_sub(1, Ordering::Relaxed);
        log::debug!(
            "Worker '{}': sweeping task finished, tasks left {}",
            std::thread::current().name().unwrap(),
            count
        );
        if count == 0 {
            log::debug!(
                "Worker '{}': No tasks left,finish sweeping",
                std::thread::current().name().unwrap()
            );
            self.state.store(FINISH, Ordering::Release);
        }

        log::debug!("Concurrent sweep finished.");
    }
}

pub struct SweeperWorker {
    pub queue: ArcQueue<SweeperJob>,
    pub rt_state: RcState,
    pub state: Arc<PoolState<SweeperJob>>,
}

impl SweeperWorker {
    pub fn new(
        queue: ArcQueue<SweeperJob>,
        state: Arc<PoolState<SweeperJob>>,
        rt_state: RcState,
    ) -> Self {
        Self {
            queue,
            state,
            rt_state,
        }
    }
}

impl WorkerTrait<SweeperJob> for SweeperWorker {
    fn state(&self) -> &PoolState<SweeperJob> {
        &self.state
    }

    fn queue(&self) -> &ArcQueue<SweeperJob> {
        &self.queue
    }

    fn process_job(&mut self, job: SweeperJob) {
        job.perform(&self.rt_state)
    }
}

pub struct SweepPool {
    state: Arc<PoolState<SweeperJob>>,
}

impl SweepPool {
    pub fn new(threads: usize) -> Self {
        assert!(
            threads > 0,
            "Sweeping pools require at least a single thread"
        );
        Self {
            state: Arc::new(PoolState::new(threads)),
        }
    }
    /// Schedules a job onto the global queue.
    pub fn schedule(&self, job: SweeperJob) {
        self.state.push_global(job);
    }
    /// Informs this pool it should terminate as soon as possible.
    pub fn terminate(&self) {
        self.state.terminate();
    }

    /// Starts the pool, without blocking the calling thread.
    pub fn start(&self, vm_state: RcState) -> JoinList<()> {
        let handles = self
            .state
            .queues
            .iter()
            .enumerate()
            .map(|(index, queue)| self.spawn_thread(index, queue.clone(), vm_state.clone()))
            .collect();

        JoinList::new(handles)
    }

    fn spawn_thread(
        &self,
        id: usize,
        queue: ArcQueue<SweeperJob>,
        rt_state: RcState,
    ) -> thread::JoinHandle<()> {
        let state = self.state.clone();
        log::warn!("Spawn Sweep Worker {}", id);
        thread::Builder::new()
            .name(format!("Sweeper-{}", id))
            .spawn(move || {
                SweeperWorker::new(queue, state, rt_state).run();
            })
            .unwrap()
    }
}
