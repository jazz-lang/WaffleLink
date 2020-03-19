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

use super::space::*;
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
#[derive(Copy, Clone)]
pub struct Slot {
    value: CellPointer,
    slot: *mut CellPointer,
}

impl Slot {
    pub fn from_ptr(p: *const CellPointer) -> Self {
        assert!(!p.is_null());
        Self {
            value: unsafe { *p },
            slot: p as *mut CellPointer,
        }
    }

    pub fn set(&self, addr: CellPointer) {
        unsafe {
            *self.slot = addr;
        }
    }
}

pub struct CopyingCollector {
    pub space: Space,
    pub needs_gc: bool,
    pub stack: Vec<Slot>,
    pub remembered_permanent: std::collections::HashSet<CellPointer>,
    pub rootset: Vec<*mut RootedInner>,
}

impl CopyingCollector {
    pub fn new(page_size: usize) -> Self {
        Self {
            space: Space::new(page_size),
            needs_gc: false,
            stack: vec![],
            remembered_permanent: Default::default(),
            rootset: vec![],
        }
    }

    pub fn copy(&mut self) {
        log::debug!("--Copying GC started--");
        let mut new_space = Space::new(self.space.page_size);
        /*while let Some(slot) = self.stack.pop() {
            let ptr = self.copy_object(&slot, &mut new_space);
            if slot.value.get().color != CELL_BLACK
                && !self
                    .remembered_permanent
                    .contains(&(slot.value.raw.raw as usize))
            {
                if !slot.value.is_permanent() {
                    slot.value.get_mut().color = CELL_BLACK;
                } else {
                    self.remembered_permanent
                        .insert(slot.value.raw.raw as usize);
                }
                slot.value.get().trace(|pointer| {
                    let slot = Slot::from_ptr(pointer);
                    slot.value.get_mut().color = CELL_GREY;
                    self.stack.push(slot);
                });
            }
        }*/
        while let Some(slot) = self.stack.pop() {
            if slot.value.is_permanent() && self.remembered_permanent.contains(&slot.value) {
                continue;
            } else if slot.value.is_permanent() {
                slot.value.get().trace(|cb| {
                    self.stack.push(Slot::from_ptr(cb));
                });
                self.remembered_permanent.insert(slot.value);
            } else if !slot.value.is_marked() {
                slot.value.mark(true);
                self.copy_object(&slot, &mut new_space);
            }
        }

        // finalization:
        for page in self.space.pages.iter() {
            /*log::trace!(
                "Sweeping memory page from {:p} to {:p} (memory page limit is {:p})",
                page.data.to_ptr::<u8>(),
                page.top.to_ptr::<u8>(),
                page.limit.to_ptr::<u8>()
            );*/
            let end = page.top;
            let mut scan = page.data;
            while scan < end {
                let cell_ptr = scan.to_mut_ptr::<Cell>();
                let cell = CellPointer {
                    raw: crate::util::tagged::TaggedPointer::new(cell_ptr),
                };
                if cell.get().color == CELL_WHITE_A {
                    log::trace!("Sweep {:p} '{}'", cell_ptr, cell);
                    unsafe {
                        std::ptr::drop_in_place(cell_ptr);
                    }
                }

                scan = scan.offset(std::mem::size_of::<Cell>());
            }
        }
        log::debug!("--Copying GC finished--");
        self.needs_gc = false;
        self.space.clear();
        self.space = new_space;
    }

    fn copy_object(&mut self, slot: &Slot, to_space: &mut Space) -> CellPointer {
        if slot.value.is_permanent() {
            return slot.value;
        }
        if slot.value.is_marked() && to_space.contains(slot.value.get().forward) {
            slot.set(slot.value.get().forward.to_cell());
            return slot.value.get().forward.to_cell();
        } else {
            let new_ptr = to_space.allocate(std::mem::size_of::<Cell>(), &mut false);
            log::trace!(
                "Copy {:p}->{:p}",
                slot.value.raw.raw,
                new_ptr.to_ptr::<u8>(),
            );
            slot.value.get().copy_to_addr(new_ptr);
            slot.value.get_mut().forward = new_ptr;

            let cell = new_ptr.to_cell();
            slot.set(cell);
            cell.get_mut().color = CELL_WHITE_A;
            return cell;
        }
    }
}

impl HeapTrait for CopyingCollector {
    fn should_collect(&self) -> bool {
        self.needs_gc
    }

    fn enable(&mut self) {}
    fn disable(&mut self) {}
    fn is_enabled(&self) -> bool {
        true
    }

    fn trace_process(&mut self, proc: &Arc<crate::runtime::process::Process>) {
        let channel = proc.local_data().channel.lock();
        channel.trace(|pointer| {
            self.stack.push(Slot::from_ptr(pointer));
        });
        proc.trace(|pointer| {
            self.stack.push(Slot::from_ptr(pointer));
        });
        let mut stack = vec![];
        self.rootset.retain(|elem_raw| unsafe {
            let elem = &**elem_raw;
            if elem.rooted.load(Ordering::Acquire) {
                stack.push(Slot::from_ptr(&elem.inner));
                true
            } else {
                let _ = Box::from_raw(*elem_raw);
                false
            }
        });
        self.stack.extend(&stack);
    }

    fn allocate(&mut self, _: &Arc<Process>, _: GCType, cell: Cell) -> RootedCell {
        //log::debug!("Allocate cell");
        let mut needs_gc = false;
        let ptr = self
            .space
            .fast_allocate(std::mem::size_of::<Cell>(), &mut needs_gc)
            .to_mut_ptr::<Cell>();
        unsafe {
            ptr.write(cell);
        }
        if needs_gc {
            log::debug!("gc needed");
        }
        self.needs_gc = needs_gc;
        let ptr = CellPointer {
            raw: crate::util::tagged::TaggedPointer::new(ptr),
        };
        let raw = Box::into_raw(Box::new(RootedInner {
            inner: ptr,
            rooted: AtomicBool::new(true),
        }));

        self.rootset.push(raw);
        RootedCell { inner: raw }
    }

    fn collect_garbage(
        &mut self,
        proc: &Arc<crate::runtime::process::Process>,
    ) -> Result<(), bool> {
        self.stack.clear();
        self.trace_process(proc);
        self.copy();
        Ok(())
    }
}
