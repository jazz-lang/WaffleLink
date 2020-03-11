//! Serial mark&sweep collector.
//!
//! This collector implements mark-sweep scheme and allocates memory by
//! using default global allocator.

use super::*;
use std::alloc::{alloc, dealloc, Layout};

pub struct SerialCollector {
    heap: Vec<CellPointer>,
    bytes_allocated: usize,
    threshold: usize,
    stack: Vec<CellPointer>,
    enabled: bool,
}

impl SerialCollector {
    pub fn new(threshold: usize) -> Self {
        Self {
            heap: vec![],
            bytes_allocated: 0,
            threshold,
            stack: vec![],
            enabled: true,
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
            if !value.is_marked() {
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

    fn allocate(&mut self, _: &Arc<Process>, _: GCType, cell: Cell) -> CellPointer {
        self.alloc(cell)
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
        })
    }
}
