use super::block_directory::*;
use super::markedblock::*;
use super::precise_allocation::*;
use super::*;
use intrusive_collections::{LinkedList, UnsafeRef};
use std::collections::HashSet;
use std::collections::LinkedList as List;
pub type HeapVersion = u32;

/// SIZE_STEP is synonym for ATOM_SIZE.
pub const SIZE_STEP: usize = ATOM_SIZE;
/// Sizes up to this amount get a size class for each size step.
pub const PRECISE_CUTOFF: usize = 80;
/// The amount of available payload in a block is the block's size minus the footer.
pub const BLOCK_PAYLOAD: usize = PAYLOAD_SIZE;

/// The largest cell we're willing to allocate in a MarkedBlock the "normal way" (i.e. using size
/// classes, rather than a large allocation) is half the size of the payload, rounded down. This
/// ensures that we only use the size class approach if it means being able to pack two things
/// into one block.
pub const LARGE_CUTOFF: usize = (BLOCK_PAYLOAD / 2) & !(SIZE_STEP - 1);

/// We have an extra size class for size zero.
pub const NUM_SIZE_CLASSES: usize = LARGE_CUTOFF / SIZE_STEP + 1;

/// The version of freshly allocated blocks.
pub const NULL_VERSION: HeapVersion = 0;
/// The version that the heap starts out with. Set to make sure that next_version(NULL_VERSION) != INITIAL_VERSION.
pub const INITIAL_VERSION: HeapVersion = 2;

#[inline]
pub fn next_version(mut version: HeapVersion) -> HeapVersion {
    version = version.wrapping_add(1);
    if version == NULL_VERSION {
        version = INITIAL_VERSION;
    }
    version
}

pub const fn size_class_to_index(size_class: usize) -> usize {
    (size_class + SIZE_STEP - 1) / SIZE_STEP
}

pub const fn index_to_size_class(index: usize) -> usize {
    index * SIZE_STEP
}

pub fn optimal_size_for(bytes: usize) -> usize {
    if bytes <= PRECISE_CUTOFF {
        super::round_up_to_multiple_of(SIZE_STEP, bytes)
    } else if bytes <= LARGE_CUTOFF {
        SIZE_CLASSES_FOR_SIZE_STEP[size_class_to_index(bytes)]
    } else {
        bytes
    }
}

pub struct MarkedSpace {
    pub(crate) capacity: usize,

    pub(crate) precise_allocation_set: HashSet<*mut PreciseAllocation>,
    pub(crate) precise_allocations: Vec<*mut PreciseAllocation>,

    pub(crate) precise_allocations_nursery_offset: u32,
    pub(crate) precise_allocations_offset_for_this_collection: u32,
    pub(crate) precise_allocations_nursery_offset_for_sweep: u32,
    pub(crate) precise_allocations_for_this_collection_size: u32,
    pub(crate) precise_allocations_for_this_collection_begin: *mut *mut PreciseAllocation,
    pub(crate) precise_allocations_for_this_collection_end: *mut *mut PreciseAllocation,
    pub(crate) marking_version: HeapVersion,
    pub(crate) newly_allocated_version: HeapVersion,

    pub(crate) is_iterating: bool,
    pub(crate) is_marking: bool,
    pub(crate) directory_lock: parking_lot::Mutex<()>,
    pub(crate) directories: List<BlockDirectory>,
}

pub static SIZE_CLASSES_FOR_SIZE_STEP: once_cell::sync::Lazy<[usize; NUM_SIZE_CLASSES]> =
    once_cell::sync::Lazy::new(|| {
        let mut result = [0; NUM_SIZE_CLASSES];
        build_size_class_table(&mut result, |x| x, |x| x);

        result
    });

pub fn size_classes() -> Vec<usize> {
    let mut result = vec![];
    if super::GC_LOG {
        eprintln!("Block size: {}", BLOCK_SIZE);
        eprintln!("Footer size: {}", FOOTER_SIZE);
    }

    let mut add = |vec: &mut Vec<usize>, size_class| {
        let size_class = round_up_to_multiple_of(ATOM_SIZE, size_class);
        if super::GC_LOG {
            eprintln!("--Adding MarkedSpace size class: {}", size_class);
        }
        vec.push(size_class);
    };

    let mut size = SIZE_STEP;
    while size < PRECISE_CUTOFF {
        add(&mut result, size);
        size += SIZE_STEP;
    }

    if GC_LOG {
        eprintln!("---Marked block payload size: {}", BLOCK_PAYLOAD);
    }

    for i in 0.. {
        let approximate_size = (PRECISE_CUTOFF as f64 * 1.4f64.powi(i)) as usize;

        if approximate_size > LARGE_CUTOFF {
            break;
        }
        let size_class = round_up_to_multiple_of(SIZE_STEP, approximate_size);
        if GC_LOG {
            eprintln!("---Size class: {}", size_class);
        }

        let cells_per_block = BLOCK_PAYLOAD / size_class;
        let possibly_better_size_class = (BLOCK_PAYLOAD / cells_per_block) & !(SIZE_STEP - 1);
        if GC_LOG {
            eprintln!(
                "---Possibly better size class: {}",
                possibly_better_size_class
            );
        }
        let original_wastage = BLOCK_PAYLOAD - cells_per_block * size_class;
        let new_wastage = (possibly_better_size_class - size_class) * cells_per_block;
        if GC_LOG {
            eprintln!(
                "---Original wastage: {}, new wastage: {}",
                original_wastage, new_wastage
            );
        }

        let better_size_class = if new_wastage > original_wastage {
            size_class
        } else {
            possibly_better_size_class
        };
        eprintln!("---Choosing size class: {}", better_size_class);
        if better_size_class == *result.last().unwrap() {
            continue;
        }

        if better_size_class > LARGE_CUTOFF || better_size_class > 100000 {
            break;
        }
        add(&mut result, better_size_class);
    }
    add(&mut result, 256);
    result.sort_unstable();
    result.dedup();
    if GC_LOG {
        eprintln!("--Heap MarkedSpace size class dump: {:?}", result);
    }

    result
}

pub fn build_size_class_table(
    table: &mut [usize],
    cons: impl Fn(usize) -> usize,
    dcons: impl Fn(usize) -> usize,
) {
    let mut next_index = 0;
    for size_class in size_classes() {
        let entry = cons(size_class);
        let index = size_class_to_index(size_class);
        for i in next_index..=index {
            table[i] = entry;
        }
        next_index = index + 1;
    }
    for i in next_index..NUM_SIZE_CLASSES {
        table[i] = dcons(index_to_size_class(i));
    }
}

impl MarkedSpace {
    pub fn sweep_precise_allocations(&mut self) {
        let mut src_index = self.precise_allocations_nursery_offset_for_sweep as usize;
        let mut dst_index = src_index;
        while src_index < self.precise_allocations.len() {
            let allocation = self.precise_allocations[src_index];
            src_index += 1;
            unsafe {
                (&mut *allocation).sweep();
                if (&*allocation).is_empty() {
                    self.capacity -= (&*allocation).cell_size();
                    (&mut *allocation).destroy();
                    continue;
                }
                (&mut *allocation).index_in_space = dst_index as u32;
                self.precise_allocations[dst_index] = allocation;
                dst_index += 1;
            }
        }
        self.precise_allocations.shrink_to_fit();
        self.precise_allocations_nursery_offset = self.precise_allocations.len() as u32;
    }

    pub fn new() -> Self {
        Self {
            marking_version: INITIAL_VERSION,
            newly_allocated_version: NULL_VERSION,
            directories: List::new(),
            directory_lock: parking_lot::Mutex::new(()),
            precise_allocations: Vec::new(),
            precise_allocation_set: HashSet::new(),
            precise_allocations_for_this_collection_begin: 0 as *mut _,
            precise_allocations_for_this_collection_size: 0,
            precise_allocations_for_this_collection_end: 0 as *mut _,
            precise_allocations_offset_for_this_collection: 0,
            precise_allocations_nursery_offset: 0,
            precise_allocations_nursery_offset_for_sweep: 0,
            is_iterating: false,
            is_marking: false,
            capacity: 0,
        }
    }

    pub fn end_marking(&mut self) {
        if next_version(self.newly_allocated_version) == INITIAL_VERSION {
            self.directories.iter().for_each(|directory| {
                directory.blocks.iter().for_each(|block| unsafe {
                    (&**block).block().reset_allocated();
                });
            });
        }

        self.newly_allocated_version = next_version(self.newly_allocated_version);


        for i in self.precise_allocations_offset_for_this_collection as usize..self.precise_allocations.len() {
            unsafe {
                (&mut*self.precise_allocations[i as usize]).is_newly_allocated = false;
            }
        }

        self.directories.iter().for_each(|directory| {
            directory.blocks.iter().for_each(|block| {
                let b = unsafe {&mut **block};
               // TODO
            });
        });


        self.is_marking = false;
    }
}
