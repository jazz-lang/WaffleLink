use super::waker::*;
use core::cell::UnsafeCell;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
pub struct Executor {
    receiver: flume::Receiver<Arc<Task>>,
    sender: flume::Sender<Arc<Task>>,
    terminate: AtomicBool,
}

impl Executor {
    pub fn new() -> Self {
        let (sender, receiver) = flume::unbounded();
        Self {
            sender,
            receiver,
            terminate: AtomicBool::new(false),
        }
    }
    pub async fn spawn_async(&self, task: impl Future<Output = ()> + Send + 'static) {
        let future = Box::pin(task);
        let task = Arc::new(Task {
            future: UnsafeCell::new(Some(future)),
            task_sender: self.sender.clone(),
        });
        match self.sender.send_async(task).await {
            Ok(_) => (),
            _ => unreachable!(),
        }
    }

    pub fn spawn(&self, task: impl Future<Output = ()> + Send + 'static) {
        let future = Box::pin(task);
        let task = Arc::new(Task {
            future: UnsafeCell::new(Some(future)),
            task_sender: self.sender.clone(),
        });
        match self.sender.send(task) {
            Ok(_) => (),
            _ => unreachable!(),
        }
    }
    pub fn block_on(&self, task: impl Future<Output = ()> + Send + 'static) {
        let future = Box::pin(task);
        let task = Arc::new(Task {
            future: UnsafeCell::new(Some(future)),
            task_sender: self.sender.clone(),
        });
        loop {
            unsafe {
                let mut future_slot = &mut *task.future.get();
                if let Some(mut future) = future_slot.take() {
                    let waker = waker_ref(&task);
                    let context = &mut Context::from_waker(&*waker);
                    // `BoxFuture<T>` is a type alias for
                    // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
                    // We can get a `Pin<&mut dyn Future + Send + 'static>`
                    // from it by calling the `Pin::as_mut` method.
                    let result = future.as_mut().poll(context);
                    /*if let Poll::Pending = future.as_mut().poll(context) {
                        // We're not done processing the future, so put it
                        // back in its task to be run again in the future.
                        *future_slot = Some(future);
                    }*/
                    match result {
                        Poll::Pending => *future_slot = Some(future),
                        Poll::Ready(_) => break,
                    }
                }
            }
        }
    }
    pub fn terminate(&self) {
        self.terminate.store(true, Ordering::Relaxed);
    }
    pub fn run(&self) {
        unsafe {
            while !self.terminate.load(Ordering::Relaxed) {
                if let Ok(task) = self.receiver.try_recv() {
                    // Take the future, and if it has not yet completed (is still Some),
                    // poll it in an attempt to complete it.
                    let mut future_slot = &mut *task.future.get();
                    if let Some(mut future) = future_slot.take() {
                        // Create a `LocalWaker` from the task itself
                        let waker = waker_ref(&task);
                        let context = &mut Context::from_waker(&*waker);
                        // `BoxFuture<T>` is a type alias for
                        // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
                        // We can get a `Pin<&mut dyn Future + Send + 'static>`
                        // from it by calling the `Pin::as_mut` method.
                        if let Poll::Pending = future.as_mut().poll(context) {
                            // We're not done processing the future, so put it
                            // back in its task to be run again in the future.
                            *future_slot = Some(future);
                        }
                    }
                }
            }
            // complete all uncompleted tasks
            while let Ok(task) = self.receiver.try_recv() {
                // Take the future, and if it has not yet completed (is still Some),
                // poll it in an attempt to complete it.
                let mut future_slot = &mut *task.future.get();
                if let Some(mut future) = future_slot.take() {
                    // Create a `LocalWaker` from the task itself
                    let waker = waker_ref(&task);
                    let context = &mut Context::from_waker(&*waker);
                    // `BoxFuture<T>` is a type alias for
                    // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
                    // We can get a `Pin<&mut dyn Future + Send + 'static>`
                    // from it by calling the `Pin::as_mut` method.
                    if let Poll::Pending = future.as_mut().poll(context) {
                        // We're not done processing the future, so put it
                        // back in its task to be run again in the future.
                        *future_slot = Some(future);
                    }
                }
            }
        }
    }
}

trait Pendable {
    fn is_pending(&self) -> bool;
}

pub struct Task {
    pub future: UnsafeCell<Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,
    task_sender: flume::Sender<Arc<Task>>,
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}
impl Woke for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let this = arc_self.clone();
        match arc_self.task_sender.send(this) {
            Ok(_) => (),
            _ => unreachable!(),
        }
    }
}

pub struct Spawner {
    task_sender: flume::Sender<Arc<Task>>,
}
