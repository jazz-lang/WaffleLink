use super::{Task, TaskId};
use crossbeam_queue::ArrayQueue;
use std::collections::VecDeque;
use std::task::Waker;

pub struct Executor {
    channel: (flume::Receiver<Task>, flume::Sender<Task>),
    task_queue: VecDeque<Task>,
}
impl Executor {
    pub fn new() -> Self {
        Executor {
            task_queue: VecDeque::new(),
            channel: flume::unbounded(),
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Some(task) = self.channel.1.recv() {
                
            }
        }
    }
}
