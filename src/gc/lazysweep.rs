use super::block::*;
use super::block_set::BlockSet;
use super::precise_allocation::*;
use super::*;
use crate::isolate::Isolate;
use intrusive_collections::{intrusive_adapter, LinkedList, LinkedListLink, UnsafeRef};
use std::collections::HashSet;
intrusive_adapter!(AllocLink = UnsafeRef<LocalAllocator> : LocalAllocator {
    link: LinkedListLink
});
use std::sync::atomic::{AtomicU32, Ordering as A};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum CollectionScope {
    Full,
    Minor,
}

/// Directory of blocks of the same cell size
pub struct Directory {
    blocks: Vec<*mut BlockHeader>,
    cell_size: usize,
}

/// Allocator for single size class
pub struct LocalAllocator {
    link: LinkedListLink,
    directory: *mut Directory,
    current_block: *mut BlockHeader,
}

impl LocalAllocator {
    /// Allocate memory from current block or find unswept block, sweep it
    /// and try to allocate from it, if allocation fails request new block
    pub fn allocate(&mut self, heap: &mut LazySweepGC) -> Address {
        unsafe {
            if !self.current_block.is_null() && (&*self.current_block).unswept {
                (&mut *self.current_block).sweep();
            }
            if self.current_block.is_null() {
                return self.allocate_slow(heap);
            }
            let result = (&mut *self.current_block).allocate();

            if result.is_null() {
                return self.allocate_slow(heap);
            }
            heap.bytes_allocated_this_cycle += (&mut *self.current_block).cell_size() as usize;
            result
        }
    }

    fn allocate_slow(&mut self, heap: &mut LazySweepGC) -> Address {
        unsafe {
            let dir = &mut *self.directory;
            let mut ptr = Address::null();
            for i in 0..dir.blocks.len() {
                if dir.blocks[i] == self.current_block {
                    continue;
                }
                if (&*dir.blocks[i]).can_allocate {
                    ptr = (&mut *dir.blocks[i]).allocate();
                    if ptr.is_non_null() {
                        heap.bytes_allocated_this_cycle += (&*dir.blocks[i]).cell_size() as usize;
                        self.current_block = dir.blocks[i];
                        break;
                    }
                } else if (&*dir.blocks[i]).unswept {
                    (&mut *dir.blocks[i]).sweep();
                    ptr = (&mut *dir.blocks[i]).allocate();

                    if ptr.is_non_null() {
                        heap.bytes_allocated_this_cycle += (&*dir.blocks[i]).cell_size() as usize;
                        self.current_block = dir.blocks[i];
                        break;
                    }
                }
            }
            let res = if self.current_block.is_null() || dir.blocks.is_empty() || ptr.is_null() {
                let block = Block::new(dir.cell_size);
                dir.blocks.push(block as *mut _);
                self.current_block = block as *mut _;
                heap.blocks.add(block.block);
                let res = block.allocate();
                heap.bytes_allocated_this_cycle += block.cell_size() as usize;
                res
            } else {
                ptr
            };

            assert!(res.is_non_null());
            res
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
        super::round_up_to_multiple_of(SIZE_STEP, bytes)
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
    if super::GC_LOG {
        println!("Block size: {}", BLOCK_SIZE);
        println!("Footer size: {}", FOOTER_SIZE);
    }

    let mut add = |vec: &mut Vec<usize>, size_class| {
        let size_class = round_up_to_multiple_of(ATOM_SIZE, size_class);
        if super::GC_LOG {
            println!("--Adding MarkedSpace size class: {}", size_class);
        }
        vec.push(size_class);
    };

    let mut size = SIZE_STEP;
    while size < PRECISE_CUTOFF {
        add(&mut result, size);
        size += SIZE_STEP;
    }

    if GC_LOG {
        println!("---Marked block payload size: {}", BLOCK_PAYLOAD);
    }

    for i in 0.. {
        let approximate_size = (PRECISE_CUTOFF as f64 * 1.4f64.powi(i)) as usize;

        if approximate_size > LARGE_CUTOFF {
            break;
        }
        let size_class = round_up_to_multiple_of(SIZE_STEP, approximate_size);
        if GC_LOG {
            println!("---Size class: {}", size_class);
        }

        let cells_per_block = BLOCK_PAYLOAD / size_class;
        let possibly_better_size_class = (BLOCK_PAYLOAD / cells_per_block) & !(SIZE_STEP - 1);
        if GC_LOG {
            println!(
                "---Possibly better size class: {}",
                possibly_better_size_class
            );
        }
        let original_wastage = BLOCK_PAYLOAD - cells_per_block * size_class;
        let new_wastage = (possibly_better_size_class - size_class) * cells_per_block;
        if GC_LOG {
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
        if GC_LOG {
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
    if GC_LOG {
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
        println!("{}", dcons(index_to_size_class(i)));
        table[i] = dcons(index_to_size_class(i));
    }
}

use parking_lot::{lock_api::RawMutex, RawMutex as Lock};

/// Lazy sweep GC.
pub struct LazySweepGC {
    allocator_for_size_step: [*mut LocalAllocator; NUM_SIZE_CLASSES],
    directories: Vec<Box<Directory>>,
    pub(crate) precise_allocation_set: HashSet<*mut PreciseAllocation>,
    pub(crate) precise_allocations: Vec<*mut PreciseAllocation>,
    local_allocators: LinkedList<AllocLink>,
    scopes: *mut LocalScopeInner,
    persistent: crate::utils::segmented_vec::SegmentedVec<*mut GcBox<()>>,

    lock: Lock,
    ndefers: AtomicU32,
    isolate: *mut Isolate,
    blocks: BlockSet,

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
    ram_size: usize,
    /// Mark stack for write barrier
    mark_stack: Vec<*mut GcBox<()>>,
}
fn proportional_heap_size(heap_size: usize) -> usize {
    (heap_size as f64 * 1.27) as usize
}
impl LazySweepGC {
    /// Create new GC instance
    pub fn new() -> Self {
        Self {
            ndefers: AtomicU32::new(0),
            lock: Lock::INIT,
            allocator_for_size_step: [0 as *mut LocalAllocator; NUM_SIZE_CLASSES],
            directories: vec![],
            precise_allocation_set: HashSet::with_capacity(0),
            precise_allocations: vec![],
            local_allocators: LinkedList::new(AllocLink::new()),
            scopes: core::ptr::null_mut(),

            persistent: crate::utils::segmented_vec::SegmentedVec::with_chunk_size(32),
            isolate: core::ptr::null_mut(),
            blocks: BlockSet::new(),
            collection_scope: None,
            should_do_full_collection: false,
            max_eden_size: 8 * 1024,
            max_heap_size: 32 * 1024,
            bytes_allocated_this_cycle: 0,
            size_after_last_collect: 0,
            size_after_last_eden_collect: 0,
            size_after_last_full_collect: 0,
            size_before_last_eden_collect: 0,
            size_before_last_full_collect: 0,
            min_bytes_per_cycle: 1024 * 1024,
            ram_size: 8 * 1024 * 1024 * 1024,
            total_bytes_visited: 0,
            total_bytes_visited_this_cycle: 0,
            mark_stack: vec![],
        }
    }

    fn allocator_for(&mut self, size: usize) -> Option<*mut LocalAllocator> {
        if size <= LARGE_CUTOFF {
            let index = size_class_to_index(size);
            let alloc = self.allocator_for_size_step.get(index);
            if let Some(alloc) = alloc {
                if !alloc.is_null() {
                    return Some(*alloc);
                } else {
                    return self.allocator_for_slow(size);
                }
            } else {
                return self.allocator_for_slow(size);
            }
        } else {
            None
        }
    }

    fn allocator_for_slow(&mut self, size: usize) -> Option<*mut LocalAllocator> {
        let index = size_class_to_index(size);
        let size_class = SIZE_CLASSES_FOR_SIZE_STEP.get(index).copied();
        let size_class = if size_class.is_none() {
            return None;
        } else {
            size_class.unwrap()
        };
        let alloc = self.allocator_for_size_step[index];
        if alloc.is_null() == false {
            return Some(alloc);
        }
        if GC_LOG {
            eprintln!(
                "Creating BlockDirectory/LocalAllocator for size class: {}",
                size_class
            );
        }

        let mut directory = Box::new(Directory {
            cell_size: size_class,
            blocks: Vec::new(),
        });
        let raw = &mut *directory as *mut Directory;
        let local = LocalAllocator {
            directory: raw,
            link: LinkedListLink::new(),
            current_block: 0 as *mut _,
        };
        self.directories.push(directory);
        self.local_allocators
            .push_back(UnsafeRef::from_box(Box::new(local)));
        let last =
            self.local_allocators.back_mut().get().unwrap() as *const LocalAllocator as *mut _;
        self.allocator_for_size_step[index] = last;
        Some(last)
    }
    /// Allocate raw memory of `size` bytes.
    pub unsafe fn allocate_raw(&mut self, size: usize) -> Address {
        self.lock.lock();
        self.collect_if_necessary(true);
        // this will be executed always if size <= LARGE_CUTOFF
        if let Some(alloc) = self.allocator_for(size) {
            let res = (&mut *alloc).allocate(self);
            //self.bytes_allocated += size;
            self.lock.unlock();
            return res;
        }

        // should not be executed if size > LARGE_CUTOFF
        let res = self.allocate_slow(size);
        self.bytes_allocated_this_cycle += size;
        self.lock.unlock();
        res
    }
    /// Allocate raw memory of `size` bytes.
    pub unsafe fn allocate_raw_no_gc(&mut self, size: usize) -> Address {
        let lock = self.lock.lock();
        // this will be executed always if size <= LARGE_CUTOFF
        if let Some(alloc) = self.allocator_for(size) {
            let res = (&mut *alloc).allocate(self);
            self.lock.unlock();
            return res;
        }

        // should not be executed if size > LARGE_CUTOFF
        let res = self.allocate_slow(size);
        self.lock.unlock();
        res
    }
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

    unsafe fn collect(&mut self, full: bool, locked: bool) {
        if self.ndefers.load(A::Acquire) > 0 {
            if GC_LOG {
                eprintln!("--GC is deferred, can't start it");
            }
            return;
        }

        if !locked {
            self.lock.lock();
        }
        self.will_start_collection();
        if let Some(CollectionScope::Full) = self.collection_scope {
            self.mark_stack.clear();
            self.precise_allocations.iter().for_each(|precise| {
                (&mut **precise).flip();
            });
            for dir in self.directories.iter_mut() {
                for block in dir.blocks.iter() {
                    (&mut **block).bitmap.clear_all();
                }
            }
        }
        if GC_LOG {
            eprintln!("--start marking cycle");
        }
        let mut task = MarkingTask {
            gc: self,
            bytes_visited: 0,
            gray: Default::default(),
        };

        let start = std::time::Instant::now();
        task.run(full);
        let end = start.elapsed();
        eprintln!("--marking finished");
        if GC_LOG_TIMINGS {
            eprintln!("---done in {}ns", end.as_nanos());
        }
        let visited = task.bytes_visited;
        drop(task);
        for local in self.local_allocators.iter() {
            let local = local as *const LocalAllocator as *mut LocalAllocator;
            let local = &mut *local;
            local.current_block = 0 as *mut _;
        }

        self.update_object_counts(visited);
        if let Some(CollectionScope::Full) = self.collection_scope {
            if GC_LOG {
                eprintln!("--start full sweep cycle");
            }
            let start = std::time::Instant::now();
            let this = &mut *(self as *mut Self);
            for dir in self.directories.iter_mut() {
                dir.blocks.retain(|block| {
                    let b = &mut **block;
                    if b.sweep() {
                        if GC_LOG {
                            eprintln!(
                                "--destroy block {:p}, cell size: {}",
                                b.block,
                                b.cell_size()
                            );
                        }
                        this.blocks.remove(b.block);
                        b.destroy();
                        false
                    } else {
                        true
                    }
                })
            }
            if GC_LOG {
                eprintln!("--full sweep done in {}ns", start.elapsed().as_nanos());
            }
        } else {
            for dir in self.directories.iter_mut() {
                dir.blocks.iter().for_each(|block| {
                    let block = &mut **block;
                    block.unswept = true;
                    block.can_allocate = !block.freelist.is_empty();
                });
            }
        }

        self.precise_allocations.retain(|precise| {
            let alloc = &mut **precise;
            if !alloc.is_marked() {
                if GC_LOG {
                    eprintln!(
                        "--sweep precise allocation {:p}, size: {}",
                        alloc,
                        alloc.cell_size()
                    );
                }
                alloc.destroy();
                false
            } else {
                true
            }
        });
        self.update_allocation_limits();

        self.lock.unlock();
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
    unsafe fn collect_if_necessary(&mut self, locked: bool) {
        if self.bytes_allocated_this_cycle <= self.max_eden_size {
            return;
        }
        self.collect(false, locked);
    }

    unsafe fn allocate_slow(&mut self, size: usize) -> Address {
        if size <= LARGE_CUTOFF {
            panic!("FATAL: attampting to allocate small object using large allocation.\nreqested allocation size: {}",size);
        }

        let size = round_up_to_multiple_of(16, size);
        assert_ne!(size, 0);
        let allocation = PreciseAllocation::try_create(size, self.precise_allocations.len() as _);
        self.precise_allocations.push(allocation);
        Address::from_ptr((&*allocation).cell())
    }
    /// Mark if this cell is unmarked.
    pub fn test_and_set_marked(&mut self, cell: *mut GcBox<()>) -> bool {
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

use std::collections::VecDeque;
struct MarkingTask<'a> {
    gc: &'a mut LazySweepGC,
    gray: VecDeque<*mut GcBox<()>>,
    bytes_visited: usize,
}
impl<'a> MarkingTask<'a> {
    pub fn run(&mut self, full: bool) {
        if full {}
        self.process_roots();
        self.process_gray();
    }
    fn process_roots(&mut self) {
        unsafe {
            let this = self as *mut Self;
            let this = &mut *this;
            /*self.gc.scopes.retain(|scope| {
                if (&**scope).dead {
                    let _ = Box::from_raw(*scope);
                    false
                } else {
                    (&mut **scope).locals.retain(|local| {
                        if local.is_null() {
                            false
                        } else {
                            this.mark(*local);
                            true
                        }
                    });
                    true
                }
            });*/
            let mut head = self.gc.scopes;
            while !head.is_null() {
                let prev = (&*head).prev;
                (&mut *head).locals.retain(|local| {
                    if local.is_null() {
                        false
                    } else {
                        this.mark(*local);
                        true
                    }
                });
                head = prev;
            }
        }
    }

    fn process_gray(&mut self) {
        while let Some(item) = self.gc.mark_stack.pop() {
            unsafe {
                let obj = &mut *item;
                if obj.header.cell_state == GC_WHITE {
                    obj.header.cell_state = GC_GRAY;
                    self.gray.push_back(item);
                }
            }
        }
        while let Some(item) = self.gray.pop_front() {
            unsafe {
                (&mut *item).header.cell_state = GC_BLACK;
            }
            self.visit_value(item);
        }
    }

    fn visit_value(&mut self, val: *mut GcBox<()>) {
        let obj = unsafe { &mut *val };

        obj.trait_object().visit_references(&mut |item| {
            self.mark(item as *mut _);
        });
    }
    fn mark(&mut self, object: *mut GcBox<()>) {
        let obj = unsafe { &mut *object };

        if !self.gc.test_and_set_marked(object) {
            if GC_VERBOSE_LOG {
                eprintln!("---mark {:p}", object);
            }
            obj.header.cell_state = GC_GRAY;
            self.bytes_visited += obj.trait_object().size() + core::mem::size_of::<Header>();
            self.gray.push_back(object);
        }
    }
}

impl super::GarbageCollector for LazySweepGC {
    fn set_isolate(&mut self, isolate: *mut crate::isolate::Isolate) {
        self.isolate = isolate;
    }
    unsafe fn local_scopes(&mut self) -> *mut LocalScopeInner {
        self.scopes
    }

    fn last_local_scope(&mut self) -> Option<UndropLocalScope> {
        if self.scopes.is_null() {
            None
        } else {
            Some(UndropLocalScope { inner: self.scopes })
        }
    }

    fn new_local_scope(&mut self) -> LocalScope {
        let mut scope = Box::into_raw(Box::new(LocalScopeInner {
            prev: core::ptr::null_mut(),
            next: self.scopes,
            gc: self as *mut Self,
            locals: crate::utils::linked_list::LinkedList::with_capacity(1),
            dead: false,
        }));
        unsafe {
            (&mut *scope).locals.set_chunk_size(16);
        }
        if !self.scopes.is_null() {
            unsafe {
                (&mut *self.scopes).prev = scope;
            }
        }
        self.scopes = scope;
        LocalScope { inner: scope }
    }
    fn defer_gc(&mut self) {
        self.ndefers.fetch_add(1, A::AcqRel);
    }

    fn undefer_gc(&mut self) {
        self.ndefers.fetch_sub(1, A::AcqRel);
    }
    fn full(&mut self) {
        unsafe {
            self.collect(true, false);
        }
    }

    fn minor(&mut self) {
        unsafe {
            self.collect(false, false);
        }
    }

    fn allocate(&mut self, size: usize) -> Address {
        let res = unsafe { self.allocate_raw(size) };
        res
    }

    fn allocate_no_gc(&mut self, size: usize) -> Address {
        let res = unsafe { self.allocate_raw_no_gc(size) };

        res
    }

    fn write_barrier(&mut self, object: *mut GcBox<()>, field: *mut GcBox<()>) {
        unsafe {
            let obj = &mut *object;
            if obj.header.cell_state != GC_BLACK {
                if (&*field).header.cell_state != GC_WHITE {
                    return;
                }
            }
            if GC_VERBOSE_LOG {
                eprintln!("WriteBarrier: {:p}<-{:p}", object, field);
            }
            obj.header.cell_state = GC_GRAY;
            self.mark_stack.push(object);
        }
    }
}
