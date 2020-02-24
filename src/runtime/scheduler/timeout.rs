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

//! Processes suspended with a timeout.
use crate::runtime::process::Process;
use crate::util::arc::Arc;
use std::cmp;
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};
use std::vec::Vec;

/// A process that should be resumed after a certain point in time.
pub struct Timeout {
    /// The time after which the timeout expires.
    resume_after: Instant,
}

impl Timeout {
    pub fn new(suspend_for: Duration) -> Self {
        Timeout {
            resume_after: Instant::now() + suspend_for,
        }
    }

    pub fn with_rc(suspend_for: Duration) -> Arc<Self> {
        Arc::new(Self::new(suspend_for))
    }

    pub fn remaining_time(&self) -> Option<Duration> {
        let now = Instant::now();

        if now >= self.resume_after {
            None
        } else {
            Some(self.resume_after - now)
        }
    }
}

/// A Timeout and a Process to store in the timeout heap.
///
/// Since the Timeout is also stored in a process we can't also store a Process
/// in a Timeout, as this would result in cyclic references. To work around
/// this, we store the two values in this separate TimeoutEntry structure.
struct TimeoutEntry {
    timeout: Arc<Timeout>,
    process: Arc<Process>,
}

impl TimeoutEntry {
    pub fn new(process: Arc<Process>, timeout: Arc<Timeout>) -> Self {
        TimeoutEntry { process, timeout }
    }

    fn is_valid(&self) -> bool {
        self.process.is_suspended_with_timeout(&self.timeout)
    }

    fn acquire_rescheduling_rights(&self) -> bool {
        self.process.acquire_rescheduling_rights().are_acquired()
    }
}

impl PartialOrd for TimeoutEntry {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimeoutEntry {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        // BinaryHeap pops values starting with the greatest value, but we want
        // values with the smallest timeouts. To achieve this, we reverse the
        // sorting order for this type.
        self.timeout
            .resume_after
            .cmp(&other.timeout.resume_after)
            .reverse()
    }
}

impl PartialEq for TimeoutEntry {
    fn eq(&self, other: &Self) -> bool {
        self.timeout.resume_after == other.timeout.resume_after && self.process == other.process
    }
}

impl Eq for TimeoutEntry {}

/// A collection of processes that are waiting with a timeout.
///
/// This structure uses a binary heap for two reasons:
///
/// 1. At the time of writing, no mature and maintained timer wheels exist for
///    Rust. The closest is tokio-timer, but this requires the use of tokio.
/// 2. Binary heaps allow for arbitrary precision timeouts, at the cost of
///    insertions being more expensive.
///
/// All timeouts are also stored in a hash map, making it cheaper to invalidate
/// timeouts at the cost of potentially keeping invalidated entries around in
/// the heap for a while.
pub struct Timeouts {
    /// The timeouts of all processes, sorted from shortest to longest.
    timeouts: BinaryHeap<TimeoutEntry>,
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::len_without_is_empty))]
impl Timeouts {
    pub fn new() -> Self {
        Timeouts {
            timeouts: BinaryHeap::new(),
        }
    }

    pub fn insert(&mut self, process: Arc<Process>, timeout: Arc<Timeout>) {
        self.timeouts.push(TimeoutEntry::new(process, timeout));
    }

    pub fn len(&self) -> usize {
        self.timeouts.len()
    }

    pub fn remove_invalid_entries(&mut self) -> usize {
        let mut removed = 0;
        let new_heap = self
            .timeouts
            .drain()
            .filter(|entry| {
                if entry.is_valid() {
                    true
                } else {
                    removed += 1;
                    false
                }
            })
            .collect();

        self.timeouts = new_heap;

        removed
    }

    pub fn processes_to_reschedule(&mut self) -> (Vec<Arc<Process>>, Option<Duration>) {
        let mut reschedule = Vec::new();
        let mut time_until_expiration = None;

        while let Some(entry) = self.timeouts.pop() {
            if !entry.is_valid() {
                continue;
            }

            if let Some(duration) = entry.timeout.remaining_time() {
                self.timeouts.push(entry);

                time_until_expiration = Some(duration);

                // If this timeout didn't expire yet, any following timeouts
                // also haven't expired.
                break;
            }

            if entry.acquire_rescheduling_rights() {
                reschedule.push(entry.process);
            }
        }

        (reschedule, time_until_expiration)
    }
}
