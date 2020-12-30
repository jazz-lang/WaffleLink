pub mod bitmap;
pub mod block;
pub mod block_directory;
pub mod block_set;
pub mod constants;
pub mod precise_allocation;
pub mod sweeper;
pub mod tiny_bloom_filter;
use crate::heap::*;
use crate::utils::{segmented_vec::*, *};
use block::*;
use block_directory::BlockDirectory;
use constants::*;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct LazyMarkSweep {}
impl LazyMarkSweep {
    /// Mark if this cell is unmarked.
    pub fn test_and_set_marked(cell: *mut RawGc) -> bool {
        unsafe {
            let c = &mut *cell;
            if c.is_precise_allocation() {
                (&mut *c.precise_allocation()).test_and_set_marked()
            } else {
                let block = c.block();
                let header = (&*block).header();

                header.test_and_set_marked(Address::from_ptr(cell))
            }
        }
    }
}

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
/// Converts size class to index
pub const fn size_class_to_index(size_class: usize) -> usize {
    (size_class + SIZE_STEP - 1) / SIZE_STEP
}
/// Converts index to size class
pub const fn index_to_size_class(index: usize) -> usize {
    index * SIZE_STEP
}
/// Return optimal allocation size
pub fn optimal_size_for(bytes: usize) -> usize {
    if bytes <= PRECISE_CUTOFF {
        round_up_to_multiple_of(SIZE_STEP, bytes)
    } else if bytes <= LARGE_CUTOFF {
        SIZE_CLASSES_FOR_SIZE_STEP[size_class_to_index(bytes)]
    } else {
        bytes
    }
}

/// Size classes for size step

pub static SIZE_CLASSES_FOR_SIZE_STEP: once_cell::sync::Lazy<[usize; NUM_SIZE_CLASSES]> =
    once_cell::sync::Lazy::new(|| {
        let mut result = [0; NUM_SIZE_CLASSES];
        build_size_class_table(&mut result, |x| x, |x| x);

        result
    });

/// All size classes
pub fn size_classes() -> Vec<usize> {
    let mut result = vec![];
    if false {
        println!("Block size: {}", BLOCK_SIZE);
        println!("Footer size: {}", FOOTER_SIZE);
    }

    let add = |vec: &mut Vec<usize>, size_class| {
        let size_class = round_up_to_multiple_of(ATOM_SIZE, size_class);
        if false {
            println!("--Adding MarkedSpace size class: {}", size_class);
        }
        vec.push(size_class);
    };

    let mut size = SIZE_STEP;
    while size < PRECISE_CUTOFF {
        add(&mut result, size);
        size += SIZE_STEP;
    }

    if false {
        println!("---Marked block payload size: {}", BLOCK_PAYLOAD);
    }

    for i in 0.. {
        let approximate_size = (PRECISE_CUTOFF as f64 * 1.4f64.powi(i)) as usize;

        if approximate_size > LARGE_CUTOFF {
            break;
        }
        let size_class = round_up_to_multiple_of(SIZE_STEP, approximate_size);
        if false {
            println!("---Size class: {}", size_class);
        }

        let cells_per_block = BLOCK_PAYLOAD / size_class;
        let possibly_better_size_class = (BLOCK_PAYLOAD / cells_per_block) & !(SIZE_STEP - 1);
        if false {
            println!(
                "---Possibly better size class: {}",
                possibly_better_size_class
            );
        }
        let original_wastage = BLOCK_PAYLOAD - cells_per_block * size_class;
        let new_wastage = (possibly_better_size_class - size_class) * cells_per_block;
        if false {
            println!(
                "---Original wastage: {}, new wastage: {}",
                original_wastage, new_wastage
            );
        }

        let better_size_class = if new_wastage > original_wastage {
            size_class
        } else {
            possibly_better_size_class
        };
        if false {
            println!("---Choosing size class: {}", better_size_class);
        }
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
    if false {
        println!("--Heap MarkedSpace size class dump: {:?}", result);
    }

    result
}
/// Build size class table
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

pub struct GlobalAllocator {
    directories: Vec<Box<BlockDirectory>>,
    local_allocators: Mutex<Vec<Arc<ThreadLocalAllocator>>>,
}

impl GlobalAllocator {
    pub fn new() -> Self {
        let mut directories = Vec::with_capacity(SIZE_CLASSES_FOR_SIZE_STEP.len());
        for sz in SIZE_CLASSES_FOR_SIZE_STEP.iter() {
            directories.push(Box::new(BlockDirectory::new(*sz)));
        }
        Self {
            directories,
            local_allocators: Mutex::new(Vec::new()),
        }
    }

    pub fn new_local_allocator(self: &Arc<Self>) -> Arc<ThreadLocalAllocator> {
        let mut alloc = ThreadLocalAllocator {
            local_allocators: Vec::with_capacity(self.directories.len()),
            global: self.clone(),
        };

        for dir in self.directories.iter() {
            alloc.local_allocators.push(UnsafeCell::new(LocalAllocator {
                directory: unsafe { &*(&**dir as *const BlockDirectory) },
                current_block: null_mut(),
            }));
        }
        let t = Arc::new(alloc);
        self.local_allocators.lock().push(t.clone());

        t
    }
}

pub struct LocalAllocator {
    directory: &'static BlockDirectory,
    current_block: *mut BlockHeader,
}

impl LocalAllocator {
    pub fn allocate(&mut self) -> Address {
        unsafe {
            if self.current_block.is_null() {
                return self.allocate_slow();
            }
            let addr = (&mut *self.current_block).allocate();
            if addr.is_null() {
                return self.allocate_slow();
            }
            addr
        }
    }

    fn allocate_slow(&mut self) -> Address {
        // First try to receive block from unswept or reclaim list.
        // NOTE: retrieve_block_for_allocation could allocate new block.
        let block = self.directory.retrieve_block_for_allocation();
        if block.can_allocate {
            let addr = block.allocate();
            self.current_block = block as *mut _;
            return addr;
        }
        let block = self.directory.new_block();
        let addr = block.allocate();
        self.current_block = block as *mut _;
        return addr;
    }
}

use std::cell::UnsafeCell;
pub struct ThreadLocalAllocator {
    local_allocators: Vec<UnsafeCell<LocalAllocator>>,
    global: Arc<GlobalAllocator>,
}

impl ThreadLocalAllocator {
    pub fn allocate_small(&self, size: usize) -> Address {
        for local in self.local_allocators.iter() {
            // only one thread can access local allocator but ThreadLocalAllocator is in Arc so
            // we can't borrow it as mutable.
            let local = unsafe { &mut *local.get() };
            if local.directory.cell_size >= size as u32 {
                return local.allocate();
            }
        }
        crate::utils::unreachable();
    }
}

impl<'a> Tracer for MarkingTask<'a> {
    fn trace(&mut self, reference: *const *mut RawGc) {
        unsafe {
            let ptr = *reference;
            self.mark_cell(ptr);
        }
    }
}

pub struct MarkingTask<'a> {
    heap: &'a mut LazyMarkSweep,
    mark_stack: SegmentedVec<*mut RawGc>,
    bytes_visited: usize,
}

impl<'a> MarkingTask<'a> {
    fn mark_cell(&mut self, base: *mut RawGc) {
        if LazyMarkSweep::test_and_set_marked(base) {
            return;
        }
        unsafe { (&mut *base).set_tag(GC_GRAY) };
        self.bytes_visited += unsafe { (&*base).object_size() };
        self.mark_stack.push(base);
    }

    fn visit_children(&mut self, cell: *mut RawGc) {
        unsafe {
            (&mut *cell).set_tag(GC_BLACK);
            (&mut *cell).as_dyn().visit_references(self);
        }
    }

    fn process_markstack(&mut self) {
        while let Some(cell) = self.mark_stack.pop() {
            self.visit_children(cell);
        }
    }
}
