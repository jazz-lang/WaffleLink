use super::space::*;
use crate::runtime;
use crate::util;
pub mod remember_set;
use runtime::cell::*;
use util::mem::*;
use util::tagged::*;

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Copy, Debug)]
pub enum GenerationalGCType {
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
    old_space: Space,
    needs_gc: GenerationalGCType,
    mark_stack: Vec<Slot>,
    rootset: Vec<Slot>,
    current: GenerationalGCType,
    old_set: remember_set::RemembrSet,
    intermediate_set: remember_set::RemembrSet,
    old2intermediate_set: remember_set::RemembrSet,
}

impl GenerationalHeap {
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
            GenerationalGCType::Old => self.old_space.contains(Address::from_ptr(cell.raw.raw)),
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

    fn mark_live(&mut self) {
        let mut stack = vec![];
        if self.current == GenerationalGCType::Young {
            for value in self.old_set.iter() {
                if !value.is_marked() {
                    value.mark(true);
                    stack.push(*value);
                }
            }

            for value in self.intermediate_set.iter() {
                if !value.is_marked() {
                    value.mark(true);
                    stack.push(*value);
                }
            }
        } else if self.current == GenerationalGCType::Intermediate {
            for value in self.old2intermediate_set.iter() {
                if !value.is_marked() {
                    value.mark(true);
                    stack.push(*value);
                }
            }
        }
        for root in self.rootset.iter() {
            if self.in_current_space(root.value) {
                if !root.value.is_marked() {
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
                        cell.mark(true);
                        stack.push(cell);
                    }
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
                GenerationalGCType::Old => self.old_space.pages.iter().copied().collect::<Vec<_>>(),
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

    fn relocate(&mut self) {
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
                GenerationalGCType::Old => self.old_space.pages.iter().copied().collect::<Vec<_>>(),
            }
        };
        for page in pages {
            self.walk_heap(page.data, page.top, |_gc, object, address| {
                if object.is_marked() {
                    let dest = object.get().forward;
                    if address != dest {
                        object.get().copy_to_addr(dest);
                    }

                    let dest_obj = dest.to_cell();
                    dest_obj.mark(false);
                } else {
                    unsafe {
                        std::ptr::drop_in_place(object.raw.raw);
                    }
                }
            });
        }
    }
    /// Copy all objects from *nursery* to *intermediate* space.
    pub fn scavenge(&mut self) {
        self.mark_live();
        self.compute_forward_scavenge();
        self.update_references();
        self.relocate();
    }
}
