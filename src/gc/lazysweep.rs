use super::block::*;
use super::precise_allocation::*;
use super::*;
use intrusive_collections::{intrusive_adapter, LinkedList, LinkedListLink, UnsafeRef};
use std::collections::HashSet;

intrusive_adapter!(AllocLink = UnsafeRef<LocalAllocator> : LocalAllocator {
    link: LinkedListLink
});

pub struct Directory {
    blocks: Vec<*mut BlockHeader>,
    cell_size: usize,
}

pub struct LocalAllocator {
    link: LinkedListLink,
    directory: *mut Directory,
    current_block: *mut BlockHeader,
}

impl LocalAllocator {
    pub fn allocate(&mut self) -> Address {
        unsafe {
            if self.current_block.is_null() {
                return self.allocate_slow();
            }
            let result = (&mut *self.current_block).allocate();
            #[cold]
            if result.is_null() {
                return self.allocate_slow();
            }

            result
        }
    }

    fn allocate_slow(&mut self) -> Address {
        unsafe {
            let dir = &mut *self.directory;
            let mut ptr = Address::null();
            for i in 0..dir.blocks.len() {
                if (&*dir.blocks[i]).can_allocate {
                    
                    ptr = (&mut*dir.blocks[i]).allocate();
                    if ptr.is_non_null() {
                        self.current_block = dir.blocks[i];
                        break;
                    }
                } else if (&*dir.blocks[i]).unswept {
                    
                    (&mut *dir.blocks[i]).sweep();
                    ptr = (&mut*dir.blocks[i]).allocate();
                    if ptr.is_non_null() {
                        self.current_block = dir.blocks[i];
                        break;
                    }
                }
            }
            let res = if self.current_block.is_null() || dir.blocks.is_empty() || ptr.is_null() {
                let block = Block::new(dir.cell_size);
                dir.blocks.push(block as *mut _);
                self.current_block = block as *mut _;
                let res = block.allocate();
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

pub static SIZE_CLASSES_FOR_SIZE_STEP: once_cell::sync::Lazy<[usize; NUM_SIZE_CLASSES]> =
    once_cell::sync::Lazy::new(|| {
        let mut result = [0; NUM_SIZE_CLASSES];
        build_size_class_table(&mut result, |x| x, |x| x);

        result
    });

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

pub struct LazySweepGC {
    allocator_for_size_step: [*mut LocalAllocator; NUM_SIZE_CLASSES],
    directories: Vec<Box<Directory>>,
    pub(crate) precise_allocation_set: HashSet<*mut PreciseAllocation>,
    pub(crate) precise_allocations: Vec<*mut PreciseAllocation>,
    local_allocators: LinkedList<AllocLink>,
    scopes: Vec<*mut LocalScopeInner>,
    bytes_allocated: usize,
    bytes_allowed: usize,
}

impl LazySweepGC {
    pub fn new() -> Self {
        Self {
            allocator_for_size_step: [0 as *mut LocalAllocator; NUM_SIZE_CLASSES],
            directories: vec![],
            precise_allocation_set: HashSet::with_capacity(0),
            precise_allocations: vec![],
            local_allocators: LinkedList::new(AllocLink::new()),
            scopes: vec![],
            bytes_allocated: 0,
            bytes_allowed: 8 * 1024,
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

    pub unsafe fn allocate_raw(&mut self, size: usize) -> Address {
        // this will be executed always if size <= LARGE_CUTOFF
        if let Some(alloc) = self.allocator_for(size) {
            return (&mut *alloc).allocate();
        }
        
        // should not be executed if size > LARGE_CUTOFF
        self.allocate_slow(size)
    }

    fn collect(&mut self, full: bool) {
        if GC_LOG {
            eprintln!("--start marking cycle");
        }
        let mut task = MarkingTask {
            gc: self,
            bytes_visited: 0,
            gray: Default::default(),
        };
        let start = std::time::Instant::now();
        task.run();
        let end = start.elapsed();
        eprintln!("--marking finished");
        if GC_LOG_TIMINGS {
            eprintln!("---done in {}ns", end.as_nanos());
        }
        let visited = task.bytes_visited;
        drop(task);
        for local in self.local_allocators.iter() {
            unsafe {
                let local = local as *const LocalAllocator as *mut LocalAllocator;
                let local = &mut *local;
                local.current_block = 0 as *mut _;
            }
        }
        if full {
            if GC_LOG {
                eprintln!("--start full sweep cycle");
            }
            let start = std::time::Instant::now();
            for dir in self.directories.iter_mut() {
                dir.blocks.retain(|block| {
                    let b = unsafe { &mut **block };
                    if b.sweep() {
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
                dir.blocks.iter().for_each(|block| unsafe {
                    let block = &mut**block;
                    block.unswept = true;
                    block.can_allocate = false;
                });
            }
        }

        self.precise_allocations.retain(|precise| unsafe {
            let alloc = &mut **precise;
            if !alloc.is_marked() {
                if GC_VERBOSE_LOG {
                    eprintln!("--sweep precise allocation {:p}, size: {}",alloc,alloc.cell_size());
                }
                alloc.destroy();
                false
            } else {
                
                true
            }
        });
        self.bytes_allocated = visited;
        if self.bytes_allocated >= self.bytes_allowed {
            let prev = self.bytes_allowed;

            self.bytes_allowed = (self.bytes_allocated as f64 / 0.75) as usize;
            if GC_LOG {
                eprintln!(
                    "--Change threshold from {} bytes to {} bytes",
                    prev, self.bytes_allowed
                );
            }
        }
    }

    fn collect_if_necessary(&mut self) {
        if self.bytes_allocated > self.bytes_allowed {
            self.collect(false);
        }
    }

    unsafe fn allocate_slow(&mut self, size: usize) -> Address {
        if size <= LARGE_CUTOFF {
            panic!("FATAL: attampting to allocate small object using large allocation.\nreqested allocation size: {}",size);
        }

        let size = round_up_to_multiple_of(16, size);
        assert_ne!(size,0);
        let allocation = PreciseAllocation::try_create(size, self.precise_allocations.len() as _);
        self.precise_allocations.push(allocation);
        Address::from_ptr((&*allocation).cell())
    }

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
    pub fn run(&mut self) {
        self.gc
            .precise_allocations
            .iter()
            .for_each(|precise| unsafe {
                (&mut **precise).flip();
                
            });
        for dir in self.gc.directories.iter_mut() {
            for block in dir.blocks.iter() {
                unsafe {
                    (&mut **block).bitmap.clear_all();
                }
            }
        }
        self.process_roots();
        self.process_gray();
    }
    fn process_roots(&mut self) {
        unsafe {
            let this = self as *mut Self;
            let this = &mut *this;
            self.gc.scopes.retain(|scope| {
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
            });
        }
    }

    fn process_gray(&mut self) {
        while let Some(item) = self.gray.pop_front() {
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
                eprintln!("---mark {:p}",object);
            }
            self.bytes_visited += obj.trait_object().size() + core::mem::size_of::<Header>();
            self.gray.push_back(object);
        }
    }
}

impl super::GarbageCollector for LazySweepGC {
    fn new_local_scope(&mut self) -> LocalScope {
        let mut scope = Box::into_raw(Box::new(LocalScopeInner {
            gc: self as *mut Self,
            locals: vec![],
            dead: false
        }));
        self.scopes.push(scope);
        LocalScope {
            inner: scope
        }
    }

    fn full(&mut self) {
        self.collect(true);
    }

    fn minor(&mut self) {
        self.collect(false);
    }

    fn allocate(&mut self, size: usize) -> Address {
        let res = unsafe { self.allocate_raw(size) };
        self.bytes_allocated += size;
        res
    }
}
