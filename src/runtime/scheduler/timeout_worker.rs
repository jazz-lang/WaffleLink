use super::process_scheduler::ProcessScheduler;
use super::timeout::*;
use crate::runtime::process::Process;
use crate::util::arc::Arc;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

pub type RcProcess = Arc<Process>;

/// The maximum number of messages to process in a single timeout iteration.
const MAX_MESSAGES_PER_ITERATION: usize = 64;

enum Message {
    Terminate,
    Suspend(RcProcess, Arc<Timeout>),
}

struct TimeoutWorkerInner {
    /// The processes suspended with timeout
    timeouts: Timeouts,
    receiver: Receiver<Message>,
    /// Indicates if the timeout worker should run or terminate
    alive: bool,
}

pub struct TimeoutWorker {
    inner: UnsafeCell<TimeoutWorkerInner>,
    sender: Sender<Message>,
    expired: AtomicUsize,
}

unsafe impl Sync for TimeoutWorker {}
unsafe impl Send for TimeoutWorker {}

impl TimeoutWorker {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        let inner = TimeoutWorkerInner {
            timeouts: Timeouts::new(),
            receiver,
            alive: true,
        };

        Self {
            inner: UnsafeCell::new(inner),
            expired: AtomicUsize::new(0),
            sender,
        }
    }

    pub fn suspend(&self, process: RcProcess, duration: Duration) {
        let timeout = Timeout::with_rc(duration);
        process.suspend_with_timeout(timeout.clone());
        self.sender
            .send(Message::Suspend(process, timeout))
            .expect("Channel was closed, cannot suspend process")
    }

    pub fn terminate(&self) {
        self.sender
            .send(Message::Terminate)
            .expect("Channel was closed, cannot terminate");
    }

    pub fn increase_expired_timeouts(&self) {
        self.expired.fetch_add(1, Ordering::AcqRel);
    }

    pub fn run(&self, scheduler: &ProcessScheduler) {
        while self.is_alive() {
            self.handle_pending_messages();
            if !self.is_alive() {
                return;
            }
            let time_until_expiration = self.reschedule_expired_processes(scheduler);

            if let Some(duration) = time_until_expiration {
                self.wait_for_message_with_timeout(duration);
            } else {
                // When there are no timeouts there's no point in periodically
                // processing the list of timeouts, so instead we wait until the
                // first one is added.
                self.wait_for_message();
            }
        }
    }

    fn is_alive(&self) -> bool {
        self.inner().alive
    }
    #[allow(unused)]
    fn number_of_expired_timeouts(&self) -> f64 {
        self.expired.load(Ordering::Acquire) as f64
    }

    fn inner(&self) -> &TimeoutWorkerInner {
        unsafe { &*self.inner.get() }
    }

    #[cfg_attr(feature = "cargo-clippy", allow(clippy::mut_from_ref))]
    fn inner_mut(&self) -> &mut TimeoutWorkerInner {
        unsafe { &mut *self.inner.get() }
    }

    fn reschedule_expired_processes(&self, scheduler: &ProcessScheduler) -> Option<Duration> {
        let inner = self.inner_mut();
        let (expired, time_until_expiration) = inner.timeouts.processes_to_reschedule();

        for process in expired {
            scheduler.schedule(process);
        }

        time_until_expiration
    }

    fn handle_pending_messages(&self) {
        for message in self
            .inner_mut()
            .receiver
            .try_iter()
            .take(MAX_MESSAGES_PER_ITERATION)
        {
            self.handle_message(message);
        }
    }

    fn wait_for_message(&self) {
        let message = self
            .inner()
            .receiver
            .recv()
            .expect("Attempt to receive from a closed channel");

        self.handle_message(message);
    }

    fn wait_for_message_with_timeout(&self, wait_for: Duration) {
        if let Ok(message) = self.inner().receiver.recv_timeout(wait_for) {
            self.handle_message(message);
        }
    }

    fn handle_message(&self, message: Message) {
        let inner = self.inner_mut();

        match message {
            Message::Suspend(process, timeout) => {
                inner.timeouts.insert(process, timeout);
            }
            Message::Terminate => {
                inner.alive = false;
            }
        }
    }
}
