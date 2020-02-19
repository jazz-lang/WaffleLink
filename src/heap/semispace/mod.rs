//! Generational semispace garbage collector.
//!
//! The heap is divided into two spaces: one for old objects and second one is for young objects.
//! Young heap is larger than old heap because most objects should die young.
//!
//! This GC moves object to old heap when object generation is bigger than 5th gen.
//!
//! Current implementation does not support object finalization thus you should close IO handles or do some finalization
//! by your hand when using this GC.

use super::Space;
use super::*;
use crate::runtime::cell::*;
use crate::runtime::value::*;
use crate::util::arc::*;
use crate::util::mem::*;
use fxhash::FxBuildHasher;
use std::boxed::Box;
use std::collections::HashSet;
pub struct Heap {
    pub new_space: Space,
    pub old_space: Space,
    pub needs_gc: GCType,
    pub remembered: HashSet<usize, FxBuildHasher>,
}

impl Heap {
    pub fn new(young_page_size: usize, old_page_size: usize) -> Self {
        Self {
            new_space: Space::new(young_page_size),
            old_space: Space::new(old_page_size),
            needs_gc: GCType::None,
            remembered: HashSet::with_capacity_and_hasher(32, FxBuildHasher::default()),
        }
    }
    pub fn allocate(&mut self, tenure: GCType, cell: Cell) -> CellPointer {
        assert_ne!(tenure, GCType::None);
        let space = if tenure == GCType::Old {
            &mut self.old_space
        } else {
            &mut self.new_space
        };
        let mut needs_gc = false;
        let result = space
            .allocate(std::mem::size_of::<Cell>(), &mut needs_gc)
            .to_mut_ptr::<Cell>();
        unsafe {
            result.write(cell);
        }
        self.needs_gc = if needs_gc { tenure } else { GCType::None };
        CellPointer {
            raw: crate::util::tagged::TaggedPointer::new(result),
        }
    }
}

impl Drop for Heap {
    fn drop(&mut self) {
        self.new_space.clear();
        self.old_space.clear();
    }
}

use crate::util::tagged::*;
use intrusive_collections::{LinkedList, LinkedListLink};
pub struct GCValue {
    pub slot: *mut CellPointer,
    pub value: CellPointer,
    link: LinkedListLink,
}

impl GCValue {
    pub fn relocate(&mut self, address: CellPointer) {
        if self.slot.is_null() == false {
            unsafe {
                self.slot.write(address);
            }
        }
        if !self.value.is_marked() {
            self.value.mark(true);
            self.value.get_mut().forward = Address::from_ptr(address.raw.raw);
        }
    }
}

intrusive_adapter!(
    GCValueAdapter =  Box<GCValue> : GCValue {link: LinkedListLink}
);

pub struct GC {
    grey_items: LinkedList<GCValueAdapter>,
    black_items: LinkedList<GCValueAdapter>,
    tmp_space: Space,
    gc_ty: GCType,
    white: u8,
}

impl GC {
    pub fn new() -> Self {
        Self {
            tmp_space: Space::empty(),
            grey_items: LinkedList::new(GCValueAdapter::new()),
            black_items: LinkedList::new(GCValueAdapter::new()),
            gc_ty: GCType::None,
            white: CELL_WHITE_B,
        }
    }

    pub fn flip_white(&mut self) {
        match self.white {
            CELL_WHITE_A => self.white = CELL_WHITE_B,
            CELL_WHITE_B => self.white = CELL_WHITE_A,
            _ => unreachable!(),
        }
    }

    pub fn collect_garbage(&mut self, heap: &mut Heap) {
        self.flip_white();
        if heap.needs_gc == GCType::None {
            heap.needs_gc = GCType::Young;
        }
        self.gc_ty = heap.needs_gc;
        let space = if self.gc_ty == GCType::Young {
            heap.new_space.page_size
        } else {
            heap.old_space.page_size
        };
        heap.remembered.iter().for_each(|remembered| {
            let cell = CellPointer {
                raw: TaggedPointer::new(*remembered as *mut Cell),
            };
            cell.set_color(CELL_GREY);
        });
        log::trace!(
            "Begin {:?} space collection (current worker is '{}')",
            self.gc_ty,
            std::thread::current().name().unwrap()
        );
        let mut tmp_space = Space::new(space);
        std::mem::swap(&mut self.tmp_space, &mut tmp_space);
        self.process_grey(heap);
        let space = if self.gc_ty == GCType::Young {
            &mut heap.new_space
        } else {
            &mut heap.old_space
        };
        while let Some(item) = self.black_items.pop_back() {
            item.value.set_color(CELL_WHITE_A);
            item.value.soft_mark(false);
        }
        std::mem::swap(&mut self.tmp_space, &mut tmp_space);
        let space = if self.gc_ty == GCType::Young {
            &mut heap.new_space
        } else {
            &mut heap.old_space
        };

        for page in space.pages.iter() {
            let end = page.top;
            log::trace!(
                "Sweeping memory page from {:p} to {:p} (memory page limit is {:p})",
                page.data.to_ptr::<u8>(),
                page.top.to_ptr::<u8>(),
                page.limit.to_ptr::<u8>()
            );
            let mut scan = page.data;
            while scan < end {
                let cell_ptr = scan.to_mut_ptr::<Cell>();
                scan = scan.offset(std::mem::size_of::<Cell>());
                let cell = CellPointer {
                    raw: crate::util::tagged::TaggedPointer::new(cell_ptr),
                };

                if !cell.is_marked() {
                    log::trace!("Sweep {:p} '{}'", cell_ptr, cell);
                    unsafe {
                        std::ptr::drop_in_place(cell.raw.raw);
                    }
                }
            }
        }

        space.swap(&mut tmp_space);

        if self.gc_ty != GCType::Young || heap.needs_gc == GCType::Young {
            heap.needs_gc = GCType::None;
            log::trace!("Collection finished");
            heap.remembered.iter().for_each(|remembered| {
                let cell = CellPointer {
                    raw: TaggedPointer::new(*remembered as *mut Cell),
                };
                cell.set_color(CELL_WHITE_A);
            });
        } else {
            log::trace!("Young space collected, collecting Old space");
            // Do GC for old space.
            self.collect_garbage(heap);
            heap.remembered.iter().for_each(|remembered| {
                let cell = CellPointer {
                    raw: TaggedPointer::new(*remembered as *mut Cell),
                };
                cell.set_color(CELL_WHITE_A);
            });
        }
    }

    pub fn schedule(&mut self, ptr: *mut CellPointer) {
        self.grey_items.push_back(Box::new(GCValue {
            link: LinkedListLink::new(),
            slot: ptr as *mut CellPointer,
            value: unsafe { *ptr },
        }))
    }

    pub fn process_grey(&mut self, heap: &mut Heap) {
        while self.grey_items.is_empty() != true {
            let mut value = self.grey_items.pop_back().unwrap();
            if value.value.raw.is_null() {
                continue;
            }
            log::trace!("Process {:p}", value.value.raw.raw,);
            if !value.value.is_marked() {
                if !self.is_in_current_space(&value.value) {
                    log::trace!(
                        "{:p} is not in {:?} space (generation: {})",
                        value.value.raw.raw,
                        self.gc_ty,
                        value.value.get().generation
                    );
                    if !value.value.is_soft_marked() {
                        value.value.get().trace(|ptr| {
                            self.grey_items.push_back(Box::new(GCValue {
                                link: LinkedListLink::new(),
                                slot: ptr as *mut CellPointer,
                                value: unsafe { *ptr },
                            }))
                        });
                        if !value.value.is_permanent() {
                            value.value.soft_mark(true);
                            value.value.mark(true);
                            self.black_items.push_back(value);
                        }
                    }
                    continue;
                }
                let hvalue;
                if self.gc_ty == GCType::Young {
                    let mut gc = false;
                    hvalue = value
                        .value
                        .copy_to(&mut heap.old_space, &mut self.tmp_space, &mut gc);
                    if gc {
                        heap.needs_gc = GCType::Old;
                    }
                } else {
                    let mut gc = false;
                    hvalue = value
                        .value
                        .copy_to(&mut self.tmp_space, &mut heap.new_space, &mut gc);
                    if gc {
                        heap.needs_gc = GCType::Young;
                    }
                }
                log::trace!("Copy {:p}->{:p}", value.value.raw.raw, hvalue.raw.raw);
                value.relocate(hvalue);
                value.value.get().trace(|ptr| {
                    self.grey_items.push_back(Box::new(GCValue {
                        link: LinkedListLink::new(),
                        slot: ptr as *mut CellPointer,
                        value: unsafe { *ptr },
                    }))
                })
            } else {
                let fwd = value.value.get().forward.to_mut_ptr::<Cell>();
                value.relocate(CellPointer {
                    raw: TaggedPointer::new(fwd),
                });
            }
        }
    }

    pub fn is_in_current_space(&self, value: &CellPointer) -> bool {
        if value.is_permanent() {
            log::trace!("Found permanent object {:p}, will skip it", value.raw.raw);
            return false; // we don't want to move permanent objects
        }
        if self.gc_ty == GCType::Old {
            value.get().generation >= 5
        } else {
            value.get().generation < 5
        }
    }
}

/// Semi-space generational GC.
pub struct GenerationalCopyGC {
    pub heap: Heap,
    pub gc: GC,
    pub threshold: usize,
    pub mature_threshold: usize,
}

// after collection we want the the ratio of used/total to be no
// greater than this (the threshold grows exponentially, to avoid
// quadratic behavior when the heap is growing linearly with the
// number of `new` calls):
const USED_SPACE_RATIO: f64 = 0.7;

impl HeapTrait for GenerationalCopyGC {
    fn should_collect(&self) -> bool {
        self.heap.needs_gc == GCType::Young
            || self.heap.new_space.size >= self.threshold
            || self.heap.old_space.size >= self.mature_threshold
            || self.heap.needs_gc == GCType::Old
    }

    fn allocate(&mut self, tenure: GCType, cell: Cell) -> CellPointer {
        let cell = self.heap.allocate(tenure, cell);
        cell
    }

    fn collect_garbage(&mut self, proc: &Arc<crate::runtime::process::Process>) {
        if self.heap.new_space.size >= self.threshold {
            self.heap.needs_gc = GCType::Young;
        } else if self.heap.old_space.size >= self.mature_threshold {
            self.heap.needs_gc = GCType::Old;
        }
        let channel = proc.local_data().channel.lock();
        channel.trace(|pointer| {
            self.gc.grey_items.push_back(unsafe {
                Box::new(GCValue {
                    slot: pointer as *mut _,
                    value: *pointer,
                    link: LinkedListLink::new(),
                })
            })
        });
        proc.trace(|pointer| {
            self.gc.grey_items.push_back(unsafe {
                Box::new(GCValue {
                    slot: pointer as *mut _,
                    value: *pointer,
                    link: LinkedListLink::new(),
                })
            })
        });
        self.gc.collect_garbage(&mut self.heap);
        if (self.threshold as f64) < self.heap.new_space.allocated_size as f64 * USED_SPACE_RATIO {
            self.threshold =
                (self.heap.new_space.allocated_size as f64 / USED_SPACE_RATIO) as usize;
        }
        if (self.mature_threshold as f64) < self.heap.old_space.allocated_size as f64 * 0.5 {
            self.mature_threshold = (self.heap.old_space.allocated_size as f64 / 0.5) as usize;
        }
    }

    fn clear(&mut self) {
        self.heap.old_space.clear();
        self.heap.new_space.clear();
    }

    fn schedule(&mut self, ptr: *mut CellPointer) {
        self.gc.grey_items.push_back(Box::new(GCValue {
            value: unsafe { *ptr },
            slot: ptr,
            link: LinkedListLink::new(),
        }));
    }

    fn remember(&mut self, cell_ptr: CellPointer) {
        self.heap.remembered.insert(cell_ptr.raw.raw as usize);
    }

    fn unremember(&mut self, cell_ptr: CellPointer) {
        self.heap.remembered.remove(&(cell_ptr.raw.raw as usize));
    }

    fn trace_process(&mut self, proc: &Arc<crate::runtime::process::Process>) {
        let channel = proc.local_data().channel.lock();
        channel.trace(|pointer| {
            proc.local_data_mut()
                .heap
                .schedule(pointer as *mut CellPointer);
        });
        proc.trace(|pointer| {
            proc.local_data_mut()
                .heap
                .schedule(pointer as *mut CellPointer);
        });
    }
}
