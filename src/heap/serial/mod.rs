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

//! Serial mark&sweep collector.
//!
//! This collector implements mark-sweep scheme and allocates memory by
//! using default global allocator.

use super::*;
use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
pub struct SerialCollector {
    heap: Vec<CellPointer>,
    rootset: Vec<*mut RootedInner>,
    bytes_allocated: usize,
    threshold: usize,
    stack: Vec<CellPointer>,
    enabled: bool,
    traced_perm: HashSet<CellPointer>,
}

impl SerialCollector {
    pub fn new(threshold: usize) -> Self {
        Self {
            heap: vec![],
            bytes_allocated: 0,
            rootset: vec![],
            threshold,
            stack: vec![],
            enabled: true,
            traced_perm: HashSet::new(),
        }
    }
    pub fn alloc(&mut self, cell: Cell) -> CellPointer {
        unsafe {
            let pointer = alloc(Layout::new::<Cell>()) as *mut Cell;
            assert!(!pointer.is_null());
            pointer.write(cell);
            self.heap.push(CellPointer::from(pointer as *const Cell));
            self.bytes_allocated += std::mem::size_of::<Cell>();
            return CellPointer::from(pointer as *const Cell);
        }
    }

    pub fn collection_needed(&self) -> bool {
        self.bytes_allocated > self.threshold
    }

    pub fn increase_threshold(&mut self) {
        if self.bytes_allocated as f64 > (self.threshold as f64 * 0.7) {
            self.threshold = (self.threshold as f64 * 0.7) as usize;
        }
    }

    pub fn collect(&mut self, process: &Arc<Process>) {
        let chan_lock = process.local_data().channel.lock();
        while let Some(value) = self.stack.pop() {
            if !value.is_marked() || !self.traced_perm.contains(&value) {
                log::debug!("Mark {:p} '{}'", value.raw.raw, value);
                value.mark(true);
                value.get().trace(|elem| {
                    unsafe { self.stack.push(*elem) };
                })
            }
        }
        let mut bytes = self.bytes_allocated;

        self.heap.retain(|elem| {
            if elem.is_marked() {
                elem.mark(false);
                true
            } else {
                log::debug!("Sweep {:p} '{}'", elem.raw.raw, elem);
                unsafe {
                    std::ptr::drop_in_place(elem.raw.raw);
                    dealloc(elem.raw.raw as *mut u8, Layout::new::<Cell>());
                }
                bytes -= std::mem::size_of::<Cell>();
                false
            }
        });
        self.bytes_allocated = bytes;
        self.increase_threshold();
        drop(chan_lock);
    }
}

impl HeapTrait for SerialCollector {
    fn should_collect(&self) -> bool {
        self.collection_needed() && self.enabled
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self) {
        self.enabled = false;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn remember(&mut self, _: CellPointer) {}
    fn unremember(&mut self, _: CellPointer) {}
    fn collect_garbage(
        &mut self,
        proc: &Arc<crate::runtime::process::Process>,
    ) -> Result<(), bool> {
        self.trace_process(proc);
        self.collect(proc);
        Ok(())
    }

    fn allocate(&mut self, _: &Arc<Process>, _: GCType, cell: Cell) -> RootedCell {
        let ptr = self.alloc(cell);
        let raw = Box::into_raw(Box::new(RootedInner {
            inner: ptr,
            rooted: AtomicBool::new(true),
        }));
        self.rootset.push(raw);
        RootedCell { inner: raw }
    }

    fn trace_process(&mut self, proc: &Arc<crate::runtime::process::Process>) {
        self.stack.clear();
        let chan = proc.local_data().channel.lock();
        for elem in chan.messages.iter() {
            if elem.is_cell() {
                self.stack.push(elem.as_cell());
            }
        }

        proc.trace(|elem| unsafe {
            self.stack.push(*elem);
        });
        let mut stack = vec![];
        self.rootset.retain(|elem| unsafe {
            if (**elem).rooted.load(Ordering::Acquire) {
                stack.push((**elem).inner);
                true
            } else {
                let _ = Box::from_raw(*elem); /* cleanup memory */
                false
            }
        });
        self.stack.extend(&stack);
    }
}
