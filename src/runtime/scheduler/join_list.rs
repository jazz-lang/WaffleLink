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
