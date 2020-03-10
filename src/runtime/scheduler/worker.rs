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

use super::queue::*;
use super::state::*;

pub trait Worker<T: Send> {
    fn process_job(&mut self, job: T);
    fn queue(&self) -> &ArcQueue<T>;
    fn state(&self) -> &PoolState<T>;
    #[allow(unreachable_code)]
    fn run(&mut self) {
        while self.state().is_alive() {
            if self.process_local_jobs() {
                continue;
            }

            if self.steal_from_other_queue() {
                continue;
            }

            if self.steal_from_global_queue() {
                continue;
            }

            #[cfg(test)]
            {
                // Since this method never returns unless the pool is
                // terminated, calling this method in a test would deadlock the
                // test. To prevent this from happening we break instead of
                // sleeping when running tests.
                break;
            }

            self.park();
        }
    }

    fn park(&self) {
        self.state().park_while(|| !self.state().has_global_jobs());
    }

    /// WaffleThreades all local jobs until we run out of work.
    ///
    /// This method returns true if the worker should self terminate.
    fn process_local_jobs(&mut self) -> bool {
        loop {
            if !self.state().is_alive() {
                return true;
            }

            if let Some(job) = self.queue().pop() {
                self.process_job(job);
            } else {
                return false;
            }
        }
    }

    fn steal_from_other_queue(&self) -> bool {
        // We may try to steal from our queue, but that's OK because it's empty
        // and none of the below operations are blocking.
        for queue in &self.state().queues {
            if queue.steal_into(&self.queue()) {
                return true;
            }
        }

        false
    }

    /// Steals a single job from the global queue.
    ///
    /// This method will return `true` if a job was stolen.
    fn steal_from_global_queue(&self) -> bool {
        if let Some(job) = self.state().pop_global() {
            self.queue().push_internal(job);
            true
        } else {
            false
        }
    }
}
