use super::object::GcBox;
use intrusive_collections::{intrusive_adapter, LinkedListLink, UnsafeRef};
use std::alloc::{alloc, dealloc, Layout};
use std::sync::atomic::{AtomicBool, Ordering};

intrusive_adapter!(pub PreciseAllocationNode = UnsafeRef<PreciseAllocation> : PreciseAllocation {link: LinkedListLink});
/// Precise allocation used for large objects (>= LARGE_CUTOFF).
/// Wafflelink has a good malloc that already knows what to do for large allocations. The GC shouldn't
/// have to think about such things. That's where PreciseAllocation comes in. We will allocate large
/// objects directly using malloc, and put the PreciseAllocation header just before them. We can detect
/// when a *mut GcBox is a PreciseAllocation because it will have the ATOM_SIZE / 2 bit set.
#[repr(C)]
pub struct PreciseAllocation {
    pub link: LinkedListLink,
    pub cell_size: usize,
    pub is_marked: AtomicBool,
    pub index_in_space: u32,
    pub is_newly_allocated: bool,
    pub adjusted_alignment: bool,
    pub has_valid_cell: bool,
}

impl PreciseAllocation {
    pub const ALIGNMENT: usize = super::markedblock::ATOM_SIZE;
    pub const HALF_ALIGNMENT: usize = Self::ALIGNMENT / 2;
    pub fn is_precise(raw_ptr: *mut ()) -> bool {
        (raw_ptr as usize & Self::HALF_ALIGNMENT) != 0
    }

    pub fn from_cell(ptr: *mut GcBox<()>) -> *mut Self {
        unsafe {
            ptr.cast::<u8>()
                .offset(-(Self::header_size() as isize))
                .cast()
        }
    }
    #[inline]
    pub fn base_pointer(&self) -> *mut () {
        if self.adjusted_alignment {
            return unsafe {
                (self as *const Self as *mut ()).offset(-(Self::HALF_ALIGNMENT as isize))
            };
        } else {
            self as *const Self as *mut ()
        }
    }

    pub fn clear_marked(&self) {
        self.is_marked.store(false, Ordering::Relaxed);
    }

    pub fn is_marked(&self) -> bool {
        self.is_marked.load(Ordering::Relaxed)
    }

    pub fn test_and_set_marked(&self) -> bool {
        if self.is_marked() {
            return true;
        }
        self.is_marked
            .compare_exchange(false, true, Ordering::Release, Ordering::Relaxed)
            == Ok(false)
    }
    pub fn cell(&self) -> *mut GcBox<()> {
        unsafe {
            (self as *const Self as *mut ())
                .offset(Self::header_size() as _)
                .cast()
        }
    }

    pub fn above_lower_bound(&self, raw_ptr: *mut ()) -> bool {
        let ptr = raw_ptr;
        let begin = self.cell() as *mut ();
        ptr >= begin
    }

    pub fn below_upper_bound(&self, raw_ptr: *mut ()) -> bool {
        let ptr = raw_ptr;
        let begin = self.cell() as *mut ();
        let end = (begin as usize + self.cell_size) as *mut ();
        ptr <= (end as usize + 8) as *mut ()
    }
    pub const fn header_size() -> usize {
        (core::mem::size_of::<Self>() + Self::HALF_ALIGNMENT - 1) & !(Self::HALF_ALIGNMENT - 1)
            | Self::HALF_ALIGNMENT
    }

    pub fn contains(&self, raw_ptr: *mut ()) -> bool {
        self.above_lower_bound(raw_ptr) && self.below_upper_bound(raw_ptr)
    }

    pub fn is_live(&self) -> bool {
        self.is_marked() || self.is_newly_allocated
    }

    pub fn flip(&self) {
        self.clear_marked();
    }

    pub fn is_empty(&self) -> bool {
        !self.is_marked() && !self.is_newly_allocated
    }

    pub fn sweep(&mut self) {
        if self.has_valid_cell && !self.is_live() {
            unsafe {
                let cell = self.cell();
                std::ptr::drop_in_place((&mut *cell).trait_object());
            }
            self.has_valid_cell = false;
        }
    }

    pub fn try_create(size: usize, index_in_space: u32) -> *mut Self {
        let adjusted_alignment_allocation_size = Self::header_size() + size + Self::HALF_ALIGNMENT;
        unsafe {
            let mut space = alloc(
                Layout::from_size_align(adjusted_alignment_allocation_size, Self::ALIGNMENT)
                    .unwrap(),
            );
            let mut adjusted_alignment = false;
            if !is_aligned_for_precise_allocation(space) {
                space = space.offset(Self::HALF_ALIGNMENT as _);
                adjusted_alignment = true;
            }

            space.cast::<Self>().write(Self {
                link: LinkedListLink::new(),
                adjusted_alignment,
                is_marked: AtomicBool::new(false),
                is_newly_allocated: true,
                has_valid_cell: true,
                cell_size: size,
                index_in_space,
            });

            space.cast()
        }
    }
    pub fn cell_size(&self) -> usize {
        self.cell_size
    }
    pub fn destroy(&mut self) {
        let adjusted_alignment_allocation_size =
            Self::header_size() + self.cell_size + Self::HALF_ALIGNMENT;
        let base = self.base_pointer();
        unsafe {
            dealloc(
                base.cast(),
                Layout::from_size_align(adjusted_alignment_allocation_size, Self::ALIGNMENT)
                    .unwrap(),
            );
        }
    }
}

pub fn is_aligned_for_precise_allocation(mem: *mut u8) -> bool {
    let allocable_ptr = mem as usize;
    (allocable_ptr & (PreciseAllocation::ALIGNMENT - 1)) != 0
}