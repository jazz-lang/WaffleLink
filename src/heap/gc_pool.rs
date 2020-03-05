use crate::runtime;
use crate::runtime::scheduler;
use crate::util::arc::Arc;
use runtime::process::*;
use runtime::state::*;
use scheduler::join_list::JoinList;
use scheduler::queue::*;
use scheduler::state::*;
use scheduler::worker::Worker as WorkerTrait;
use std::thread;

pub struct Worker {
    pub queue: ArcQueue<Collection>,
    pub rt_state: RcState,
    pub state: Arc<PoolState<Collection>>,
}

impl Worker {
    pub fn new(
        queue: ArcQueue<Collection>,
        state: Arc<PoolState<Collection>>,
        rt_state: RcState,
    ) -> Self {
        Self {
            queue,
            state,
            rt_state,
        }
    }
}

impl WorkerTrait<Collection> for Worker {
    fn state(&self) -> &PoolState<Collection> {
        &self.state
    }

    fn queue(&self) -> &ArcQueue<Collection> {
        &self.queue
    }

    fn process_job(&mut self, job: Collection) {
        job.perform(&self.rt_state)
    }
}

pub struct Collection {
    process: Arc<Process>,
    #[allow(unused)]
    start_time: std::time::Instant,
}

impl Collection {
    pub fn new(process: Arc<Process>) -> Self {
        Self {
            process,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn perform(&self, state: &State) {
        let local_data = self.process.local_data_mut();
        //local_data.heap.trace_process(&self.process);
        let _ = local_data.heap.collect_garbage(&self.process);
        state.scheduler.schedule(self.process.clone());
    }
}

pub struct GcPool {
    state: Arc<PoolState<Collection>>,
}

impl GcPool {
    pub fn new(threads: usize) -> Self {
        assert!(threads > 0, "GC pools require at least a single thread");
        Self {
            state: Arc::new(PoolState::new(threads)),
        }
    }

    /// Schedules a job onto the global queue.
    pub fn schedule(&self, job: Collection) {
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
        queue: ArcQueue<Collection>,
        rt_state: RcState,
    ) -> thread::JoinHandle<()> {
        let state = self.state.clone();
        log::trace!("Spawn GC Worker {}", id);
        thread::Builder::new()
            .name(format!("GC {}", id))
            .spawn(move || {
                Worker::new(queue, state, rt_state).run();
            })
            .unwrap()
    }
}
