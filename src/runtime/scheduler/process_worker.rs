use super::queue::*;
use super::state::*;
use super::worker::*;
use crate::runtime::process::Process;
use crate::util::arc::Arc;
use rand::rngs::ThreadRng;
use rand::thread_rng;
/// The state that a worker is in.
#[derive(Eq, PartialEq, Debug)]
pub enum Mode {
    /// The worker should process its own queue or other queues in a normal
    /// fashion.
    Normal,

    /// The worker should only process a particular job, and not steal any other
    /// jobs.
    Exclusive,
}

/// A worker owned by a thread, used for executing jobs from a scheduler queue.
pub struct ProcessWorker {
    /// The unique ID of this worker, used for pinning jobs.
    pub id: usize,

    /// A randomly generated integer that is incremented upon request. This can
    /// be used as a seed for hashing. The integer is incremented to ensure
    /// every seed is unique, without having to generate an entirely new random
    /// number.
    pub random_number: u64,

    /// The random number generator for this thread.
    pub rng: ThreadRng,

    /// The queue owned by this worker.
    queue: ArcQueue<Arc<Process>>,

    /// The state of the pool this worker belongs to.
    state: Arc<PoolState<Arc<Process>>>,

    /// The mode this worker is in.
    mode: Mode,
}

impl ProcessWorker {
    /// Starts a new worker operating in the normal mode.
    pub fn new(
        id: usize,
        queue: Arc<Queue<Arc<Process>>>,
        state: Arc<PoolState<Arc<Process>>>,
    ) -> Self {
        ProcessWorker {
            id,
            random_number: rand::random(),
            rng: thread_rng(),
            queue,
            state,
            mode: Mode::Normal,
        }
    }
    /// Changes the worker state so it operates in exclusive mode.
    ///
    /// When in exclusive mode, only the currently running job will be allowed
    /// to run on this worker. All other jobs are pushed back into the global
    /// queue.
    pub fn enter_exclusive_mode(&mut self) {
        self.queue.move_external_jobs();

        while let Some(job) = self.queue.pop() {
            self.state.push_global(job);
        }

        self.mode = Mode::Exclusive;
    }
    pub fn leave_exclusive_mode(&mut self) {
        self.mode = Mode::Normal;
    }
    /// Performs a single iteration of the normal work loop.
    fn normal_iteration(&mut self) {
        if self.process_local_jobs() {
            return;
        }

        if self.steal_from_other_queue() {
            return;
        }

        if self.queue.move_external_jobs() {
            return;
        }

        if self.steal_from_global_queue() {
            return;
        }

        self.state
            .park_while(|| !self.state.has_global_jobs() && !self.queue.has_external_jobs());
    }
    /// Runs a single iteration of an exclusive work loop.
    fn exclusive_iteration(&mut self) {
        if self.process_local_jobs() {
            return;
        }

        // Moving external jobs would allow other workers to steal them,
        // starving the current worker of pinned jobs. Since only one job can be
        // pinned to a worker, we don't need a loop here.
        if let Some(job) = self.queue.pop_external_job() {
            self.process_job(job);
            return;
        }

        self.state.park_while(|| !self.queue.has_external_jobs());
    }
}

impl Worker<Arc<Process>> for ProcessWorker {
    fn state(&self) -> &PoolState<Arc<Process>> {
        &self.state
    }

    fn queue(&self) -> &ArcQueue<Arc<Process>> {
        &self.queue
    }

    fn run(&mut self) {
        while self.state.is_alive() {
            match self.mode {
                Mode::Normal => self.normal_iteration(),
                Mode::Exclusive => self.exclusive_iteration(),
            };
        }
    }
    fn process_job(&mut self, job: Arc<Process>) {
        crate::runtime::RUNTIME.run_with_error_handling(self, &job);
    }
}
