//! Cake Garbage Collector (CGC)
//!
//! The GC runs when process execution is suspended, is type accurate (aka precise).
//! It is a stop-the-world mark and sweep. It is non-generational and non-compacting.
//! Allocation is done using "pages" and allocation may use bump or freelist allocation.
//!
//! The algorithm decomposes into several steps.
//! This is a high level description of the algorithm being used.
//!
//!
//! 1. GC performs sweep termination.
//!
//!     a. Stop the world. This causes process to reach a GC safe-point.
//!
//!     b. Free all empty&unused pages to OS memory.
//!
//! 2. GC performs mark phase.
//!
//!     a. Gather root objects. This is done by scanning interpreter frames and
//!        native stack using conservative roots.
//!
//!     b. GC drains stack of grey objects, scanning each grey object to black
//!        and shading all pointers found in the object.
//!
//! 3. GC performs sweep phase.
//!
//!     a. If lazy sweep is enabled set `needs_sweep` on all pages to true, otherwise
//!        just sweep memory.
//!
//!     b. Start the world.
//!
//! ## Lazy sweep
//!
//! With lazy sweep, sweep is done only when program tries to use memory page. Lazy sweep
//! reduces STW latency by doing sweep incrementally, this makes program run faster, but
//! might introduce *very* slow pauses when allocating memory.

pub mod allocator;
pub mod freelist;
use crate::common::ptr::*;
use crate::common::space::*;
use crate::runtime;
use runtime::cell::*;
use runtime::frame::*;
use runtime::process::*;
use std::collections::VecDeque;
use std::sync::atomic::AtomicUsize;
pub struct Heap {
    pub alloc: allocator::FreeListAllocator,
    pub needs_gc: bool,
    pub gc_threshold: usize,
    pub allocated_bytes: usize,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            alloc: allocator::FreeListAllocator::new(Space::new(
                16 * 1024, /* 16 kb per page */
            )),
            needs_gc: false,
            gc_threshold: 4 * 1024, // first GC cycle at 4kb heap usage
            allocated_bytes: 0,
        }
    }

    fn gather_roots(&mut self, stack: &mut VecDeque<*const Ptr<Cell>>) {
        let local_data = local_data();

        for frame in local_data.frames.iter() {
            frame.trace(stack);
        }
    }

    pub fn collect(&mut self, stack: &mut VecDeque<*const Ptr<Cell>>) {
        self.needs_gc = false;
        // 1. Sweep termination
        for i in (0..self.alloc.space.pages.len()).rev() {
            let all_free = self.alloc.space.pages[i].sweep();
            if all_free {
                let page = self.alloc.space.pages.swap_remove(i);
                page.uncommit();
            }
        }
        // No alive objects and pages, just create a new page and return from gc cycle.
        if self.alloc.space.pages.is_empty() {
            self.alloc.init();
            return;
        }

        // 2. Root scanning
        self.gather_roots(stack);

        // 3. Marking
        self.mark(stack);
        // 4. Initialize lazy sweep.
        self.sweep();
    }

    fn sweep(&mut self) {
        for page in self.alloc.space.pages.iter_mut() {
            page.needs_sweep = true;
        }
    }

    fn mark(&mut self, stack: &mut VecDeque<*const Ptr<Cell>>) {
        while let Some(value) = stack.pop_front() {
            let p: Ptr<Ptr<Cell>> = Ptr::from_raw(value as *mut Ptr<Cell>);
            let p2 = p.get();
            if p2.color != CELL_BLACK {
                p2.color = CELL_BLACK;
                p2.trace(stack);
            }
        }
    }
    pub fn allocate(&mut self, frame: &mut Frame, cell: Cell) -> Ptr<Cell> {
        let memory = self.alloc.allocate(std::mem::size_of::<Cell>(), false);
        let memory = if memory.is_null() {
            let mut stack = VecDeque::new();
            frame.trace(&mut stack);
            self.collect(&mut stack);
            self.alloc.allocate(std::mem::size_of::<Cell>(), true)
        } else {
            memory
        };
        let raw = memory.to_mut_ptr::<Cell>();
        unsafe {
            raw.write(cell);
        }

        Ptr {
            raw: memory.to_mut_ptr::<Cell>(),
        }
    }
    pub fn allocate_cell(&mut self, cell: Cell) -> Ptr<Cell> {
        let memory = self.alloc.allocate(std::mem::size_of::<Cell>(), true);
        let raw = memory.to_mut_ptr::<Cell>();
        unsafe {
            raw.write(cell);
        }

        Ptr {
            raw: memory.to_mut_ptr::<Cell>(),
        }
    }
}
