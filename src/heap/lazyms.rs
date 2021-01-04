pub mod bitmap;
pub mod block;
pub mod block_directory;
pub mod block_set;
pub mod constants;
pub mod precise_allocation;
pub mod sweeper;
use crate::mutex::Mutex as VMutex;
use crate::safepoint::*;
use crate::utils::VolatileCell;
use precise_allocation::*;
use std::collections::BTreeSet;
pub mod tiny_bloom_filter;
use crate::heap::*;
use crate::utils::{segmented_vec::*, *};
use block::*;
use block_directory::BlockDirectory;
use constants::*;
use parking_lot::Mutex;
use std::sync::Arc;
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum CollectionScope {
    Full,
    Minor,
}
fn proportional_heap_size(heap_size: usize) -> usize {
    (heap_size as f64 * 1.27) as usize
}
pub struct LazyMarkSweep {
    pub(crate) precise_allocation_set: BTreeSet<*mut PreciseAllocation>,
    pub(crate) precise_allocations: Vec<*mut PreciseAllocation>,
    //global: Arc<GlobalAllocator>,
    collection_scope: Option<CollectionScope>,
    max_eden_size: usize,
    max_heap_size: usize,
    min_bytes_per_cycle: usize,
    size_after_last_collect: usize,
    size_after_last_full_collect: usize,
    size_before_last_full_collect: usize,
    size_before_last_eden_collect: usize,
    size_after_last_eden_collect: usize,
    bytes_allocated_this_cycle: usize,
    should_do_full_collection: bool,
    total_bytes_visited_this_cycle: usize,
    total_bytes_visited: usize,
    local_allocators: Vec<LocalAllocator>,
    blocks: *mut BlockHeader,
    gc_black: u8,
    gc_white: u8,
    write_barrier_buf: &'static mut [usize; 64],
    write_barrier_buf_cur: *mut usize,
    write_barrier_buf_end: *mut usize,
    slow_write_barrier: SegmentedVec<*mut RawGc>,
}

impl LazyMarkSweep {
    pub fn new() -> Self {
        unsafe {
            let buf = Box::leak(Box::new([0usize; 64]));
            let cur = buf.as_mut_ptr();
            let end = cur.offset(64);
            Self {
                blocks: 0 as *mut _,
                gc_black: GC_BLACK,
                gc_white: GC_WHITE,
                precise_allocation_set: BTreeSet::new(),
                precise_allocations: vec![],
                write_barrier_buf: buf,
                write_barrier_buf_cur: cur,
                write_barrier_buf_end: end,
                slow_write_barrier: SegmentedVec::with_chunk_size(128),
                local_allocators: Vec::new(),
                max_eden_size: 8 * 1024,
                max_heap_size: 32 * 1024,
                bytes_allocated_this_cycle: 0,
                size_after_last_collect: 0,
                size_after_last_eden_collect: 0,
                size_after_last_full_collect: 0,
                size_before_last_eden_collect: 0,
                size_before_last_full_collect: 0,
                min_bytes_per_cycle: 1024 * 1024,
                total_bytes_visited: 0,
                total_bytes_visited_this_cycle: 0,
                should_do_full_collection: false,
                collection_scope: None,
            }
        }
    }
}

const GC_LOG: bool = false;
impl LazyMarkSweep {
    fn should_do_full_collection(&self) -> bool {
        self.should_do_full_collection
    }
    fn will_start_collection(&mut self) {
        if self.should_do_full_collection() {
            self.collection_scope = Some(CollectionScope::Full);
            self.should_do_full_collection = false;
            if GC_LOG {
                eprintln!("FullCollection");
            }
        } else {
            self.collection_scope = Some(CollectionScope::Minor);
            if GC_LOG {
                eprintln!("EdenCollection");
            }
        }
        if let Some(CollectionScope::Full) = self.collection_scope {
            self.size_before_last_full_collect =
                self.size_after_last_collect + self.bytes_allocated_this_cycle;
        } else {
            self.size_before_last_eden_collect =
                self.size_after_last_collect + self.bytes_allocated_this_cycle;
        }
    }
    fn update_object_counts(&mut self, bytes_visited: usize) {
        if let Some(CollectionScope::Full) = self.collection_scope {
            self.total_bytes_visited = 0;
        }
        self.total_bytes_visited_this_cycle = bytes_visited;
        self.total_bytes_visited += self.total_bytes_visited_this_cycle;
    }

    fn update_allocation_limits(&mut self) {
        // Calculate our current heap size threshold for the purpose of figuring out when we should
        // run another collection. This isn't the same as either size() or capacity(), though it should
        // be somewhere between the two. The key is to match the size calculations involved calls to
        // didAllocate(), while never dangerously underestimating capacity(). In extreme cases of
        // fragmentation, we may have size() much smaller than capacity().
        let mut current_heap_size = 0;
        current_heap_size += self.total_bytes_visited;

        if let Some(CollectionScope::Full) = self.collection_scope {
            self.max_heap_size = proportional_heap_size(current_heap_size).max(32 * 1024);
            self.max_eden_size = self.max_heap_size - current_heap_size;
            self.size_after_last_full_collect = current_heap_size;
            if GC_LOG {
                eprintln!("Full: currentHeapSize = {}", current_heap_size);
                eprintln!("Full: maxHeapSize = {}\nFull: maxEdenSize = {}\nFull: sizeAfterLastFullCollect = {}",self.max_heap_size,self.max_eden_size,self.size_after_last_full_collect);
            }
        } else {
            assert!(current_heap_size >= self.size_after_last_collect);

            // Theoretically, we shouldn't ever scan more memory than the heap size we planned to have.
            // But we are sloppy, so we have to defend against the overflow.
            self.max_eden_size = if current_heap_size > self.max_heap_size {
                0
            } else {
                self.max_heap_size - current_heap_size
            };
            self.size_after_last_eden_collect = current_heap_size;
            let eden_to_old_gen_ratio = self.max_eden_size as f64 / self.max_heap_size as f64;
            let min_eden_to_old_gen_ratio = 1.0 / 3.0;
            if eden_to_old_gen_ratio < min_eden_to_old_gen_ratio {
                self.should_do_full_collection = true;
            }
            // This seems suspect at first, but what it does is ensure that the nursery size is fixed.
            self.max_heap_size += current_heap_size - self.size_after_last_collect;
            self.max_eden_size = self.max_heap_size - current_heap_size;
            if GC_LOG {
                eprintln!(
                    "Eden: eden to old generation ratio: {}\nEden: minimum eden to old generation ratio {}",
                    eden_to_old_gen_ratio,min_eden_to_old_gen_ratio
                );
                eprintln!("Eden: maxEdenSize = {}", self.max_eden_size);
                eprintln!("Eden: maxHeapSize = {}", self.max_heap_size);
                eprintln!(
                    "Eden: shouldDoFullCollection = {}",
                    self.should_do_full_collection
                );
                eprintln!("Eden: currentHeapSize = {}", current_heap_size);
            }
        }
        self.size_after_last_collect = current_heap_size;
        self.bytes_allocated_this_cycle = 0;
    }

    unsafe fn collect(&mut self, full: bool) {
        /*if !safepoint_start_gc() {
            return;
        }*/
        let lock = sweeper::Sweeper::terminate();
        //let _threads = safepoint_wait_for_the_world();
        self.will_start_collection();
        if full {
            self.collection_scope = Some(CollectionScope::Full);
        }
        /*let mut local = self.global.local_allocators.lock();
        local.iter_mut().for_each(|local| {
            local.local_allocators.iter().for_each(|x| {
                let x = &mut *x.get();
                x.current_block = 0 as *mut _;
            });
        });
        drop(local);
        */

        {
            /*// SAFETY: we can't borrow directories as mutable since they're in Arc but world
            // is stopped and only this loop mutates directories.
            for dir in self.global.directories.iter() {
                let dir = &**dir as *const BlockDirectory as *mut BlockDirectory;
                let dir = &mut *dir;
                // we want to drain all channels so after GC cycle lazy sweeping will work fine.
                let _ = dir.unswept_list_recv.drain();
                let _ = dir.reclaim_list_recv.drain();

                for item in dir.blocks.get_mut() {
                    if let Some(CollectionScope::Full) = self.collection_scope {
                        // clear all mark bitmaps in full collection.
                        (&mut **item).mark_bitmap.clear_to_zeros();
                    }
                    let _ = dir.unswept_list_snd.send(*item);
                }
            }*/
            {}
            /*if let Some(CollectionScope::Full) = self.collection_scope {
                    // Swap black and white colors when full collection happens
                    let white = self.global.gc_black.load(Ordering::Relaxed);
                    let black = self.global.gc_white.load(Ordering::Relaxed);
                    self.global.gc_black.store(black, Ordering::Relaxed);
                    self.global.gc_white.store(white, Ordering::Relaxed);
                    for alloc in self.global.local_allocators.lock().iter() {
                        // SAFETY: Only one thread has access to thread local allocator when STW cycle.
                        {
                            let alloc =
                                &**alloc as *const ThreadLocalAllocator as *mut ThreadLocalAllocator;
                            let alloc = &mut *alloc;
                            alloc.gc_white = white;
                            alloc.gc_black = black;
                        }
                    }
                }
            }*/
        }
        let mut task = MarkingTask {
            heap: self,
            bytes_visited: 0,
            mark_stack: SegmentedVec::with_chunk_size(64),
            white: 0,
            black: 0,
        };

        task.process_markstack();

        let visited = task.bytes_visited;
        self.update_object_counts(visited);

        self.update_allocation_limits();
        sweeper::Sweeper::notify(lock);

        //safepoint_end_gc();
    }
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

    #[inline]
    fn write_barrier(&mut self, object: *mut RawGc, field: *mut RawGc) {
        unsafe {
            let obj = &mut *object;
            if obj.tag() != self.gc_black {
                return;
            }
            if (&*field).tag() != self.gc_white {
                return;
            }
            (&mut *obj).set_tag(GC_GRAY);
            // fast path: write to local buffer and increment pointer
            self.write_barrier_buf_cur.write(object as _);
            self.write_barrier_buf_cur = self.write_barrier_buf_cur.offset(1);
            // if thread local buffer is full push current buffer to global buffer.
            if self.write_barrier_buf_end == self.write_barrier_buf_cur {
                self.write_barrier_slow();
                self.write_barrier_buf_cur = self.write_barrier_buf.as_ptr() as *mut _;
            }
        }
    }

    unsafe fn write_barrier_slow(&mut self) {
        for obj in self.write_barrier_buf.iter() {
            self.slow_write_barrier.push((*obj) as *mut _);
        }
        //self.global.mark_stack_lock.unlock_nogc();
    }
    #[inline]
    pub unsafe fn allocate_small(&mut self, size: usize) -> Address {
        /*for local in self.local_allocators.iter() {
            // only one thread can access local allocator but ThreadLocalAllocator is in Arc so
            // we can't borrow it as mutable.
            let local = &mut *local.get();
            if local.directory.cell_size >= size as u32 {
                return local.allocate(self as *const Self as *mut Self);
            }
        }*/
        crate::utils::unreachable();
    }
    #[inline]
    pub fn allocate(&mut self, size: usize) -> Address {
        if size <= LARGE_CUTOFF {
            return unsafe { self.allocate_small(size) };
        }
        todo!() // TODO: Large allocation
    }
}
use atomic::*;

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
    white: u8,
    black: u8,
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
            (&mut *cell).set_tag(self.black);
            (&mut *cell).as_dyn().visit_references(self);
        }
    }

    fn process_markstack(&mut self) {
        self.white = self.heap.gc_white;
        self.black = self.heap.gc_black;
        while let Some(cell) = self.mark_stack.pop() {
            self.visit_children(cell);
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
