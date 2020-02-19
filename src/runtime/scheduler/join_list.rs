/// Joining of multiple threads.
use std::thread::{JoinHandle, Result as ThreadResult};

/// A JoinList can be used to join one or more threads easily.
pub struct JoinList<T> {
    handles: Vec<JoinHandle<T>>,
}

impl<T> JoinList<T> {
    /// Creates a new JoinList that will join the given threads.
    pub fn new(handles: Vec<JoinHandle<T>>) -> Self {
        JoinList { handles }
    }

    /// Waits for all the threads to finish.
    ///
    /// The return values of the threads are ignored.
    pub fn join(self) -> ThreadResult<()> {
        for handle in self.handles {
            handle.join()?;
        }

        Ok(())
    }
}
