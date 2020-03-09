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
use crate::runtime;
use crate::runtime::scheduler;
use crate::util::arc::Arc;
use crossbeam::channel::{unbounded, Receiver, Sender};
use crossbeam::deque::{Injector, Steal, Stealer, Worker};
use runtime::cell::*;
use runtime::process::*;
use runtime::state::*;
use scheduler::join_list::JoinList;
use scheduler::queue::*;
use scheduler::state::*;
use scheduler::worker::Worker as WorkerTrait;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread;
pub struct MarkingJob {
    pub process: Arc<Process>,
    pub queue: Worker<CellPointer>,
    pub injector: Arc<Injector<CellPointer>>,
    pub state: Arc<AtomicU8>,
    pub remembered_permanent: HashSet<usize>,
    pub snd: Sender<usize>,
}

impl MarkingJob {
    pub fn perform(&mut self) {
        self.mark();
    }
    fn mark(&mut self) {
        while self.state.load(Ordering::Acquire) != MARKING {
            std::thread::yield_now();
        }
        let mut marked = 0;
        while let Some(pointer) = self.pop_job() {
            if (pointer.get().forward.atomic_load() as u8 == CELL_BLACK)
                || (pointer.is_permanent()
                    && self
                        .remembered_permanent
                        .contains(&(pointer.raw.raw as usize)))
            {
                continue;
            }
            if pointer.is_permanent() {
                self.remembered_permanent.insert(pointer.raw.raw as usize);
            } else {
                pointer.get().forward.atomic_store(CELL_BLACK as *mut u8);
            }
            marked += 1;
            log::debug!("Trace {:p} '{}'", pointer.raw.raw, pointer);
            pointer.get().trace(|ptr| {
                let ptr = unsafe { *ptr };
                self.queue.push(ptr);
            });
        }
        //self.snd.send(marked).unwrap();
        log::debug!("Concurrent marking finished.");
        self.state.store(INIT_SWEEP, Ordering::Release);
    }

    fn pop_job(&self) -> Option<CellPointer> {
        if let Some(job) = self.queue.pop() {
            return Some(job);
        }

        loop {
            match self.injector.steal_batch_and_pop(&self.queue) {
                Steal::Retry => {}
                Steal::Empty => break,
                Steal::Success(job) => return Some(job),
            };
        }

        None
    }
}

pub struct MarkingWorker {
    pub queue: ArcQueue<MarkingJob>,
    pub rt_state: RcState,
    pub state: Arc<PoolState<MarkingJob>>,
}

impl MarkingWorker {
    pub fn new(
        queue: ArcQueue<MarkingJob>,
        state: Arc<PoolState<MarkingJob>>,
        rt_state: RcState,
    ) -> Self {
        Self {
            queue,
            state,
            rt_state,
        }
    }
}

impl WorkerTrait<MarkingJob> for MarkingWorker {
    fn state(&self) -> &PoolState<MarkingJob> {
        &self.state
    }

    fn queue(&self) -> &ArcQueue<MarkingJob> {
        &self.queue
    }

    fn process_job(&mut self, mut job: MarkingJob) {
        job.perform();
    }
}

pub struct MarkingPool {
    state: Arc<PoolState<MarkingJob>>,
}

impl MarkingPool {
    pub fn new(threads: usize) -> Self {
        assert!(threads > 0, "Marking pool require at least a single thread");
        Self {
            state: Arc::new(PoolState::new(threads)),
        }
    }
    /// Schedules a job onto the global queue.
    pub fn schedule(&self, job: MarkingJob) {
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
        queue: ArcQueue<MarkingJob>,
        rt_state: RcState,
    ) -> thread::JoinHandle<()> {
        let state = self.state.clone();
        log::warn!("Spawn Marking Worker {}", id);
        thread::Builder::new()
            .name(format!("Marking-{}", id))
            .spawn(move || {
                MarkingWorker::new(queue, state, rt_state).run();
            })
            .unwrap()
    }
}
