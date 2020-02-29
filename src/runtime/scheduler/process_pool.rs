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

use super::join_list::JoinList;
use super::process_worker::ProcessWorker;
use super::queue::Queue;
use super::state::PoolState;
use super::worker::Worker;
use crate::runtime;
use crate::util::arc::Arc;
use runtime::process::Process;
use std::thread;

/// A pool of threads for running lightweight "green" processes.
///
/// A pool consists out of one or more workers, each backed by an OS thread.
/// Workers can perform work on their own as well as steal work from other
/// workers.
pub struct ProcessPool {
    pub state: Arc<PoolState<Arc<Process>>>,

    /// The base name of every thread in this pool.
    name: String,
}

impl ProcessPool {
    pub fn new(name: String, threads: usize) -> Self {
        assert!(
            threads > 0,
            "A ProcessPool requires at least a single thread"
        );

        Self {
            name,
            state: Arc::new(PoolState::new(threads)),
        }
    }

    /// Schedules a job onto a specific queue.
    pub fn schedule_onto_queue(&self, queue: usize, job: Arc<Process>) {
        self.state.schedule_onto_queue(queue, job);
    }

    /// Schedules a job onto the global queue.
    pub fn schedule(&self, job: Arc<Process>) {
        self.state.push_global(job);
    }

    /// Informs this pool it should terminate as soon as possible.
    pub fn terminate(&self) {
        self.state.terminate();
    }

    /// Starts the pool, blocking the current thread until the pool is
    /// terminated.
    ///
    /// The current thread will be used to perform jobs scheduled onto the first
    /// queue.
    pub fn start_main(&self) -> JoinList<()> {
        let join_list = self.spawn_threads_for_range(1);
        let queue = self.state.queues[0].clone();

        ProcessWorker::new(0, queue, self.state.clone()).run();

        join_list
    }

    /// Starts the pool, without blocking the calling thread.
    pub fn start(&self) -> JoinList<()> {
        self.spawn_threads_for_range(0)
    }

    /// Spawns OS threads for a range of queues, starting at the given position.
    fn spawn_threads_for_range(&self, start_at: usize) -> JoinList<()> {
        let mut handles = Vec::new();

        for index in start_at..self.state.queues.len() {
            let handle = self.spawn_thread(index, self.state.queues[index].clone());

            handles.push(handle);
        }

        JoinList::new(handles)
    }

    fn spawn_thread(&self, id: usize, queue: Arc<Queue<Arc<Process>>>) -> thread::JoinHandle<()> {
        let state = self.state.clone();

        thread::Builder::new()
            .name(format!("{} {}", self.name, id))
            .spawn(move || {
                ProcessWorker::new(id, queue, state).run();
            })
            .unwrap()
    }
}
