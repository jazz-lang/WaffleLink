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
use crate::runtime;
use crate::util;
use std::sync::atomic::{AtomicBool, Ordering};
pub mod remember_set;
use super::freelist::FreeList;
use super::freelist_alloc::FreeListAllocator;
use crate::util::arc::Arc;
use runtime::cell::*;
use runtime::process::*;
use runtime::value::*;
use util::mem::*;
use util::tagged::*;

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Copy, Debug)]
pub enum GenerationalGCType {
    None,
    Young,
    Old,
    Intermediate,
}

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

pub const INTERMEDIATE_PROMOTE_TO_OLD_FROM_GEN: usize = 3;

pub struct GenerationalHeap {
    nursery_space: Space,
    intermediate_space: Space,
    old_space: FreeListAllocator,
    needs_gc: GenerationalGCType,
    rootset: Vec<Slot>,
    current: GenerationalGCType,
    old_set: remember_set::RemembrSet,
    intermediate_set: remember_set::RemembrSet,
    tmp_space: Space,
    old2intermediate_set: remember_set::RemembrSet,
    disabled: bool,
    young_threshold: usize,
    native_roots: Vec<*mut RootedInner>,
    intermediate_threshold: usize,
}

impl GenerationalHeap {
    pub fn new(young_size: usize, old_size: usize) -> Self {
        Self {
            nursery_space: Space::new(young_size),
            intermediate_space: Space::new(old_size),
            old_space: FreeListAllocator::new(Space::new(old_size)),
            needs_gc: GenerationalGCType::None,
            current: GenerationalGCType::None,
            old_set: remember_set::RemembrSet::new(),
            intermediate_set: remember_set::RemembrSet::new(),
            tmp_space: Space::empty(),
            rootset: Vec::new(),
            old2intermediate_set: remember_set::RemembrSet::new(),
            disabled: false,
            young_threshold: 128,
            intermediate_threshold: 256,
            native_roots: vec![],
        }
    }

    /// Allocateds cell in young space. If there are not enough memory we just add page to space and set `needs_gc` to `Young`.
    pub fn allocate_young(&mut self, cell: Cell) -> CellPointer {
        let mut needs_gc = false;
        let pointer = self
            .nursery_space
            .allocate(std::mem::size_of::<Cell>(), &mut needs_gc)
            .to_mut_ptr::<Cell>();
        unsafe {
            pointer.write(cell);
        }
        if needs_gc {
            self.needs_gc = GenerationalGCType::Young;
        }

        CellPointer {
            raw: TaggedPointer::new(pointer),
        }
    }

    fn in_nursery(&self, cell: CellPointer) -> bool {
        self.nursery_space.contains(Address::from_ptr(cell.raw.raw))
    }

    fn in_current_space(&self, cell: CellPointer) -> bool {
        match self.current {
            GenerationalGCType::Intermediate => self
                .intermediate_space
                .contains(Address::from_ptr(cell.raw.raw)),
            GenerationalGCType::Young => {
                self.nursery_space.contains(Address::from_ptr(cell.raw.raw))
            }
            GenerationalGCType::Old => self
                .old_space
                .space
                .contains(Address::from_ptr(cell.raw.raw)),
            _ => unreachable!(),
        }
    }

    fn walk_heap<F>(&mut self, start: Address, end: Address, mut f: F)
    where
        F: FnMut(&mut Self, CellPointer, Address),
    {
        let mut scan = start;
        while scan < end {
            let object = scan.to_mut_ptr::<Cell>();
            let object = CellPointer {
                raw: TaggedPointer::new(object),
            };

            f(self, object, scan);

            scan = scan.offset(std::mem::size_of::<Cell>());
        }
    }

    fn mark_live(&mut self) -> usize {
        let mut live = 0;
        let mut stack = vec![];
        if self.current == GenerationalGCType::Young {
            for value in self.old_set.iter() {
                log::trace!("Trace {:p} in old2young set", value.raw.raw);
                value.get().trace(|ptr| {
                    let slot = Slot::from_ptr(ptr);
                    if self.in_current_space(slot.value) {
                        if !slot.value.is_marked() {
                            slot.value.mark(true);
                            stack.push(slot.value);
                        }
                    }
                });
            }

            for value in self.intermediate_set.iter() {
                log::trace!("Trace {:p} in intermediate2young set", value.raw.raw);
                value.get().trace(|ptr| {
                    let slot = Slot::from_ptr(ptr);
                    if self.in_current_space(slot.value) {
                        if !slot.value.is_marked() {
                            slot.value.mark(true);
                            stack.push(slot.value);
                        }
                    }
                });
            }
        } else if self.current == GenerationalGCType::Intermediate {
            for value in self.old2intermediate_set.iter() {
                log::trace!("Trace {:p} in old2intermediate set", value.raw.raw);
                value.get().trace(|ptr| {
                    let slot = Slot::from_ptr(ptr);
                    if self.in_current_space(slot.value) {
                        if !slot.value.is_marked() {
                            slot.value.mark(true);
                            stack.push(slot.value);
                        }
                    }
                });
            }
        }
        for root in self.rootset.iter() {
            if self.in_current_space(root.value) {
                if !root.value.is_marked() {
                    log::trace!(
                        "Mark root value '{}' at {:p}",
                        root.value,
                        root.value.raw.raw
                    );
                    root.value.mark(true);
                    stack.push(root.value);
                }
            }
        }
        while let Some(value) = stack.pop() {
            value.get().trace(|ptr| {
                let cell = unsafe { *ptr };
                if self.in_current_space(cell) {
                    if !cell.is_marked() {
                        live += 1;
                        log::trace!("Mark value '{}' at {:p}", cell, cell.raw.raw);
                        cell.mark(true);
                        stack.push(cell);
                    }
                }
            });
        }
        live
    }

    fn compute_forward_intermediate(&mut self) {
        let pages = {
            self.intermediate_space
                .pages
                .iter()
                .copied()
                .collect::<Vec<Page>>()
        };

        for page in pages {
            self.walk_heap(page.data, page.top, |gc, object, _| {
                if object.is_marked() {
                    let mut needs_gc = false;

                    let shall_promote = object.get().generation >= 4;
                    let fwd = if shall_promote {
                        log::debug!("Promote intermediate {:p} to old", object.raw.raw);
                        gc.old_space
                            .allocate(std::mem::size_of::<Cell>(), &mut needs_gc)
                    } else {
                        gc.tmp_space
                            .allocate(std::mem::size_of::<Cell>(), &mut false)
                    };

                    if needs_gc {
                        gc.needs_gc = GenerationalGCType::Old;
                    }

                    object.get_mut().forward = fwd;
                }
            });
        }
    }

    fn compute_forward_scavenge(&mut self) {
        let pages = {
            self.nursery_space
                .pages
                .iter()
                .copied()
                .collect::<Vec<Page>>()
        };
        for page in pages {
            self.walk_heap(page.data, page.top, |gc, object, _addr| {
                if object.is_marked() {
                    let mut needs_gc = false;
                    let fwd = gc
                        .intermediate_space
                        .allocate(std::mem::size_of::<Cell>(), &mut needs_gc)
                        .to_mut_ptr::<Cell>();
                    if needs_gc {
                        gc.needs_gc = GenerationalGCType::Intermediate;
                    }

                    object.get_mut().forward = Address::from_ptr(fwd);
                }
            });
        }
    }

    fn update_references(&mut self) {
        let pages = {
            match self.current {
                GenerationalGCType::Young => {
                    self.nursery_space.pages.iter().copied().collect::<Vec<_>>()
                }
                GenerationalGCType::Intermediate => self
                    .intermediate_space
                    .pages
                    .iter()
                    .copied()
                    .collect::<Vec<_>>(),
                _ => unreachable!(),
            }
        };

        for page in pages {
            self.walk_heap(page.data, page.top, |gc, object, _| {
                if object.is_marked() {
                    object.get().trace(|field| {
                        gc.forward_reference(Slot::from_ptr(field));
                    })
                }
            });
        }

        for root in self.rootset.iter() {
            self.forward_reference(*root);
        }
    }

    fn forward_reference(&self, slot: Slot) {
        if self.in_current_space(slot.value) {
            let fwd_addr = slot.value.get().forward;
            slot.set(CellPointer {
                raw: TaggedPointer::new(fwd_addr.to_mut_ptr()),
            });
        }
    }

    fn relocate(&mut self) -> usize {
        let pages = {
            match self.current {
                GenerationalGCType::Young => {
                    self.nursery_space.pages.iter().copied().collect::<Vec<_>>()
                }
                GenerationalGCType::Intermediate => self
                    .intermediate_space
                    .pages
                    .iter()
                    .copied()
                    .collect::<Vec<_>>(),
                _ => unreachable!(),
            }
        };
        let mut relocated = 0;
        for page in pages {
            self.walk_heap(page.data, page.top, |gc, object, address| {
                if object.is_marked() {
                    let dest = object.get().forward;
                    if address != dest {
                        relocated += 1;
                        log::trace!("Relocate {:p}->{:p}", object.raw.raw, dest.to_ptr::<u8>());
                        object.get().copy_to_addr(dest);
                    }

                    let dest_obj = dest.to_cell();
                    if gc.current != GenerationalGCType::Old {
                        dest_obj.get_mut().generation += 1;
                    }
                    dest_obj.mark(false);
                } else {
                    log::trace!(
                        "Sweep {} {:p} '{}'",
                        if gc.current == GenerationalGCType::Intermediate {
                            "intermediate"
                        } else {
                            "young"
                        },
                        object.raw.raw,
                        object
                    );
                    unsafe {
                        std::ptr::drop_in_place(object.raw.raw);
                    }
                }
            });
        }
        relocated
    }
    /// Copy all objects from *nursery* to *intermediate* space.
    pub fn scavenge(&mut self) {
        log::trace!("Scavenging started");
        self.current = GenerationalGCType::Young;
        let survived = self.mark_live();
        self.compute_forward_scavenge();
        self.update_references();
        let _survived = self.relocate();
        if survived as f64 > self.young_threshold as f64 * 0.5 {
            self.nursery_space.clear();
            self.nursery_space.page_size = align_usize(
                (self.nursery_space.page_size as f64 / 0.5) as usize,
                page_size(),
            );
            self.nursery_space.add_page(self.nursery_space.page_size);
            self.young_threshold = (self.young_threshold as f64 / 0.5) as usize;
        }
        for page in self.nursery_space.pages.iter_mut() {
            page.top = page.data;
        }
        log::trace!("Scavenging finished");
        if self.needs_gc != GenerationalGCType::Intermediate {
            self.needs_gc = GenerationalGCType::None;
        }
    }

    pub fn minor(&mut self, proc: &Arc<Process>) -> bool {
        self.current = GenerationalGCType::Intermediate;
        log::debug!("Minor cycle: mark live objects");
        self.trace_process(proc);
        let survived = self.mark_live();
        self.tmp_space = Space::empty();
        let size = if survived as f64 > self.intermediate_threshold as f64 * 0.5 {
            self.intermediate_space.page_size = align_usize(
                (self.nursery_space.page_size as f64 / 0.5) as usize,
                page_size(),
            );
            //self.nursery_space.add_page(self.nursery_space.page_size);
            self.intermediate_threshold = (self.intermediate_threshold as f64 / 0.5) as usize;
            self.intermediate_space.page_size
        } else {
            self.intermediate_space.page_size
        };
        self.tmp_space.page_size = size;
        self.tmp_space.add_page(size);
        self.compute_forward_intermediate();
        self.update_references();
        let _survived = self.relocate();
        std::mem::swap(&mut self.tmp_space, &mut self.intermediate_space);
        self.tmp_space.clear();
        let failed = self.needs_gc == GenerationalGCType::Old;
        self.needs_gc = GenerationalGCType::None;
        self.current = GenerationalGCType::None;
        failed
    }

    fn major(&mut self, proc: &Arc<Process>) {
        self.current = GenerationalGCType::Old;
        self.trace_process(proc);
        self.mark_live();
        self.sweep_old();
    }

    fn sweep_old(&mut self) {
        let mut freelist = FreeList::new();
        macro_rules! add_freelist {
            ($start: expr,$end: expr) => {
                if $start.is_non_null() {
                    let size = $end.offset_from($start);
                    freelist.add($start, size);
                }
            };
        }

        for page in self.old_space.space.pages.iter() {
            let mut garbage_start = Address::null();
            let end = page.top;
            log::trace!(
                "Sweeping memory page from {:p} to {:p} (memory page limit is {:p})",
                page.data.to_ptr::<u8>(),
                page.top.to_ptr::<u8>(),
                page.limit.to_ptr::<u8>()
            );
            let mut scan = page.data;

            while scan < end {
                let cell = scan.to_cell();

                if cell.is_marked() {
                    add_freelist!(garbage_start, scan);
                    garbage_start = Address::null();
                    cell.mark(false);
                } else if garbage_start.is_non_null() {
                    if cell.get().generation != 127 {
                        log::trace!("Sweep old {:p} '{}'", cell.raw.raw, cell);
                        unsafe {
                            std::ptr::drop_in_place(cell.raw.raw);
                        }
                    }
                    cell.get_mut().generation = 127;
                } else {
                    if cell.get().generation != 127 {
                        log::trace!("Sweep old {:p} '{}'", cell.raw.raw, cell);
                        unsafe {
                            std::ptr::drop_in_place(cell.raw.raw);
                        }
                        cell.get_mut().generation = 127;
                    }
                    garbage_start = scan;
                }
                scan = scan.offset(std::mem::size_of::<Cell>());
            }
            add_freelist!(garbage_start, page.limit);
        }
        self.old_space.freelist = freelist;
    }

    pub fn is_old(&self, cell: CellPointer) -> bool {
        self.old_space
            .space
            .contains(Address::from_ptr(cell.raw.raw))
    }

    pub fn is_intermediate(&self, cell: CellPointer) -> bool {
        self.intermediate_space
            .contains(Address::from_ptr(cell.raw.raw))
    }
    pub fn field_write_barrier_(&mut self, parent: CellPointer, child: Value) {
        if !child.is_cell() {
            return;
        }
        let cell = child.as_cell();
        if self.is_old(parent) && self.is_intermediate(cell) {
            self.old2intermediate_set.remember(parent);
        } else if self.is_old(parent) && self.in_nursery(cell) {
            self.old_set.remember(parent);
        } else if self.is_intermediate(parent) && self.in_nursery(cell) {
            self.intermediate_set.remember(parent);
        }
    }

    pub fn garbage_collect_(&mut self, proc: &Arc<Process>, minor: bool) {
        if minor {
            log::trace!("Minor GC cycle");
            log::trace!("Minor Phase 1 (Scavenge)");
            self.scavenge();
            log::trace!("Minor Phase 2 (Minor collection)");
            let do_major = self.minor(proc);
            if do_major {
                log::trace!("Promotion failed, do major collection");
                self.major(proc);
                self.old_set.prune();
                self.old2intermediate_set.prune();
                log::trace!("Finish major GC");
            }
            log::trace!("Minor collection finished");
            self.intermediate_set.prune();
        } else {
            log::trace!("Full GC triggered");
            log::trace!("Full GC: Scavenge");
            self.scavenge();
            log::trace!("Full GC: Minor");
            self.minor(proc);
            log::trace!("Full GC: Major");
            self.major(proc);
            self.old2intermediate_set.prune();
            self.old_set.prune();
            self.intermediate_set.prune();
        }
    }
}

use super::HeapTrait;

impl HeapTrait for GenerationalHeap {
    fn allocate(&mut self, proc: &Arc<Process>, _: super::GCType, cell: Cell) -> RootedCell {
        let cell = self.allocate_young(cell);
        /*if self.needs_gc == GenerationalGCType::Young {
            self.trace_process(proc);
            self.scavenge();
            if self.needs_gc == GenerationalGCType::Intermediate {
                if self.minor(proc) {
                    self.major(proc);
                }
            }
        }*/
        let raw = Box::into_raw(Box::new(RootedInner {
            rooted: AtomicBool::new(false),
            inner: cell,
        }));
        self.native_roots.push(raw);
        RootedCell { inner: raw }
    }

    fn should_collect(&self) -> bool {
        self.needs_gc != GenerationalGCType::None
    }

    fn collect_garbage(
        &mut self,
        proc: &Arc<crate::runtime::process::Process>,
    ) -> Result<(), bool> {
        if !self.disabled {
            self.trace_process(proc);
            self.garbage_collect_(
                proc,
                self.needs_gc == GenerationalGCType::Young
                    || self.needs_gc == GenerationalGCType::None
                    || self.needs_gc == GenerationalGCType::Intermediate,
            );
        }
        Ok(())
    }

    fn trace_process(&mut self, proc: &Arc<crate::runtime::process::Process>) {
        self.rootset.clear();
        let channel = proc.local_data().channel.lock();
        channel.trace(|pointer| {
            self.rootset.push(Slot::from_ptr(pointer));
        });
        proc.trace(|pointer| {
            self.rootset.push(Slot::from_ptr(pointer));
        });
        let mut rootset = vec![];
        self.native_roots.retain(|elem_raw| unsafe {
            let elem = &**elem_raw;

            if elem.rooted.load(Ordering::Acquire) {
                rootset.push(Slot::from_ptr(&elem.inner));
                true
            } else {
                let _ = Box::from_raw(*elem_raw);
                false
            }
        });
        self.rootset.extend(&rootset);
    }

    fn field_write_barrier(&mut self, parent: CellPointer, child: Value) {
        Self::field_write_barrier_(self, parent, child)
    }

    fn disable(&mut self) {
        self.disabled = true;
    }

    fn enable(&mut self) {
        self.disabled = false;
    }
    fn is_enabled(&self) -> bool {
        !self.disabled
    }
}
