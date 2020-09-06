use super::object::*;
use super::*;
use bit_set::BitSet;
pub struct BlockDirectory {
    cell_size: usize,
    blocks: Vec<*mut Block>,
    freelist: FreeList,
}

impl BlockDirectory {
    pub fn new(cell_size: usize) -> Self {
        Self {
            cell_size,
            blocks: Vec::with_capacity(4),
            freelist: FreeList::new(),
        }
    }
    pub fn prepare_for_mark(&mut self) {
        for block in self.blocks.iter() {
            unsafe {
                (&mut **block).prepare_for_mark();
            }
        }
    }

    pub fn shrink(&mut self) {
        self.blocks.retain(|block| {
            let mut b = unsafe { &mut **block };
            if b.is_empty {
                unsafe {
                    std::ptr::drop_in_place(*block);
                }
                false
            } else {
                true
            }
        })
    }
    pub fn allocate(&mut self) -> Address {
        let candidate = self.freelist.take();
        if candidate.is_null() {
            self.allocate_slow_case()
        } else {
            return candidate;
        }
    }
    pub fn sweep(&mut self) {
        for i in 0..self.blocks.len() {
            let block = self.blocks[i];
            unsafe {
                (&mut *block).sweep(&mut self.freelist);
            }
        }
    }
    pub fn find_blocks_for_allocation(&self) -> Vec<*mut Block> {
        self.blocks
            .iter()
            .filter(|x| unsafe { !(&***x).full || (&***x).is_empty })
            .copied()
            .collect()
    }
    pub fn try_allocate_in(&mut self, block: *mut Block) -> Address {
        let b = unsafe { &mut *block };
        b.sweep(&mut self.freelist);

        if self.freelist.allocation_will_fail() {
            return Address::null();
        }

        self.freelist.take()
    }

    pub fn try_allocate_without_gc(&mut self) -> Address {
        /*loop {
            let block = self.find_block_for_allocation();
            if block.is_none() {
                break;
            }
            let result = self.try_allocate_in(block.unwrap());
            if result.is_non_null() {
                return result;
            }
        }*/
        /*for block in self.find_blocks_for_allocation() {
            let result = self.try_allocate_in(block);
            if result.is_non_null() {
                return result;
            }
        }*/
        let mut this = unsafe { &mut *(self as *mut Self) };
        for block in this
            .blocks
            .iter()
            .filter(|x| unsafe { (&***x).is_empty || !(&***x).full })
        {
            let result = self.try_allocate_in(*block);
            if result.is_non_null() {
                return result;
            }
        }
        Address::null()
    }

    pub fn allocate_slow_case(&mut self) -> Address {
        let result = self.try_allocate_without_gc();
        if result.is_non_null() {
            return result;
        }

        let block = Block::boxed(self.cell_size);
        self.blocks.push(block);
        self.try_allocate_in(block)
    }
}

pub struct FreeList {
    pub head: Address,
}

impl FreeList {
    pub fn take(&mut self) -> Address {
        if self.head.is_null() {
            return self.head;
        }
        let next = unsafe { self.head.to_mut_ptr::<Address>().offset(1).read() };
        let prev = self.head;
        self.head = next;
        prev
    }

    pub fn add(&mut self, addr: Address) {
        unsafe {
            addr.to_mut_ptr::<Address>().offset(1).write(self.head);
            self.head = addr;
        }
    }

    pub fn allocation_will_fail(&self) -> bool {
        self.head.is_null()
    }

    pub const fn new() -> Self {
        Self {
            head: Address::null(),
        }
    }
}

pub const SIZE_STEP: usize = 16;
#[inline(always)]
pub const fn index_to_size_class(index: usize) -> usize {
    index * SIZE_STEP
}
#[inline(always)]
pub const fn size_class_to_index(sz: usize) -> usize {
    (sz + SIZE_STEP - 1) / SIZE_STEP
}

pub const BLOCK_SIZE: usize = 1024 * 16;

/// A per block object map.
pub struct ObjectMap {
    set: BitSet<u32>,
}

impl ObjectMap {
    /// Create a new `ObjectMap`.
    pub fn new() -> ObjectMap {
        ObjectMap {
            set: BitSet::<u32>::with_capacity(BLOCK_SIZE / 16),
        }
    }

    /// Reduce the objects address to an offset within the block.
    fn index(object: GCObjectRef) -> usize {
        (object as usize) % BLOCK_SIZE
    }

    /// Set the address as a valid object.
    pub fn set_object(&mut self, object: GCObjectRef) {
        self.set.insert(ObjectMap::index(object));
    }

    /// Unset the address as a valid object.
    pub fn unset_object(&mut self, object: GCObjectRef) {
        self.set.remove(ObjectMap::index(object));
    }

    /// Return `true` is the address is a valid object.
    pub fn is_object(&self, object: GCObjectRef) -> bool {
        self.set.contains(ObjectMap::index(object))
    }

    /// Update this `ObjectMap` with the difference of this `ObjectMap` and
    /// the other.
    pub fn difference(&mut self, other: &ObjectMap) {
        self.set.difference_with(&other.set);
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.set.clear();
    }

    /// Retrieve the values as a `HashSet`.
    pub fn as_hashset(&self, base: *mut u8) -> std::collections::HashSet<GCObjectRef> {
        self.set
            .iter()
            .map(|i| unsafe { base.offset(i as isize) as GCObjectRef })
            .collect()
    }
}
use std::alloc::{alloc, dealloc, Layout};

pub struct Block {
    freelist: FreeList,
    marks: Option<ObjectMap>,
    sz: usize,
    pub freelisted: bool,
    pub is_empty: bool,
    pub directory: *mut BlockDirectory,
    pub full: bool,
}

impl Block {
    pub fn boxed(cell_size: usize) -> *mut Self {
        unsafe {
            let mem = super::aligned_alloc(BLOCK_SIZE, BLOCK_SIZE);

            let mut mem = mem.cast::<Self>();
            mem.cast::<u8>()
                .offset(END_ATOM as _)
                .cast::<Block>()
                .write(Block {
                    is_empty: true,
                    directory: core::ptr::null_mut(),
                    full: false,
                    freelist: FreeList::new(),
                    marks: Some(ObjectMap::new()),
                    sz: cell_size,
                    freelisted: true,
                });
            let mut block = &mut *mem.cast::<u8>().offset(END_ATOM as _).cast::<Block>();
            let mut cur = block.start();
            let start = cur;
            let end = block.end();
            while cur < end {
                (&mut *cur.to_mut_ptr::<GcBox<()>>()).zap(1);
                assert!((&mut *cur.to_mut_ptr::<GcBox<()>>()).is_zapped());

                block.freelist.add(cur);
                cur = cur.offset(cell_size);
            }
            let x: u64 = core::mem::transmute([0, 1u32]);

            mem.cast::<u8>().offset(END_ATOM as _).cast::<Block>()
        }
    }
    pub fn allocate(&mut self) -> Address {
        self.is_empty = false;
        let addr = self.freelist.take();
        if addr.is_non_null() {
            addr
        } else {
            self.full = true;
            addr
        }
    }
    pub fn start(&self) -> Address {
        unsafe {
            Address::from_ptr(
                Address::from_ptr(self)
                    .to_ptr::<u8>()
                    .offset(-(END_ATOM as isize)),
            )
        }
    }

    pub fn mark(&mut self, addr: Address) {
        self.marks.as_mut().unwrap().set_object(addr.to_mut_ptr());
    }

    pub fn unmark(&mut self, addr: Address) {
        self.marks.as_mut().unwrap().unset_object(addr.to_mut_ptr());
    }

    pub fn is_marked(&self, addr: Address) -> bool {
        self.marks.as_ref().unwrap().is_object(addr.to_mut_ptr())
    }

    pub fn end(&self) -> Address {
        Address::from_ptr(self)
    }

    pub fn prepare_for_mark(&mut self) {
        self.marks.as_mut().unwrap().clear();
    }

    pub fn sweep(&mut self, freelist: &mut FreeList) {
        let mut destroy = |cell: &mut GcBox<()>| unsafe {
            if !cell.is_zapped() {
                std::ptr::drop_in_place(cell.trait_object());
                cell.zap(1);
            }
        };
        let mut handle_dead_cell = |addr: Address| unsafe {
            let object = addr.to_mut_ptr::<GcBox<()>>();
            destroy(&mut *object);
            freelist.add(addr);
        };

        let mut cur = self.start();
        let end = self.end();
        self.is_empty = true;
        let mut count = 0;

        while cur < end {
            if self.marks.as_ref().unwrap().is_object(cur.to_mut_ptr()) {
                cur = cur.offset(self.sz);
                self.is_empty = false;
                continue;
            }
            count += 1;
            handle_dead_cell(cur);
            cur = cur.offset(self.sz);
        }
        self.full = count == 0;
        self.freelisted = true;
    }
}

pub const END_ATOM: usize = (BLOCK_SIZE - core::mem::size_of::<Block>()) / SIZE_STEP;
pub const PAYLOAD_SIZE: usize = END_ATOM * SIZE_STEP;

pub const PRECISE_CUTOFF: usize = 80;

pub const LARGE_CUTOFF: usize = (BLOCK_PAYLOAD / 2) & !(SIZE_STEP - 1);
pub const BLOCK_PAYLOAD: usize = PAYLOAD_SIZE;
pub const NUM_SIZE_CLASSES: usize = LARGE_CUTOFF / SIZE_STEP + 1;

pub static SIZE_CLASSES: once_cell::sync::Lazy<Vec<usize>> = once_cell::sync::Lazy::new(|| {
    let mut result = Vec::new();
    let add = |mut size_class, result: &mut Vec<usize>| {
        size_class = round_up_to_multiple_of(SIZE_STEP, size_class);
        if result.is_empty() {
            assert_eq!(size_class, SIZE_STEP);
        }
        result.push(size_class);
    };
    // This is a definition of the size classes in our GC. It must define all of the
    // size classes from sizeStep up to largeCutoff.

    // Have very precise size classes for the small stuff. This is a loop to make it easy to reduce
    // SIZE_STEP.
    let mut size = SIZE_STEP;
    while size < PRECISE_CUTOFF {
        add(size, &mut result);
        size += SIZE_STEP;
    }
    // We want to make sure that the remaining size classes minimize internal fragmentation (i.e.
    // the wasted space at the tail end of a Block) while proceeding roughly in an exponential
    // way starting at just above the precise size classes to four cells per block.
    for i in 0usize.. {
        let approximate_size = (PRECISE_CUTOFF as f32 * ((1.4f32).powi(i as _))) as usize;

        if approximate_size > LARGE_CUTOFF {
            break;
        }

        let size_class = round_up_to_multiple_of(SIZE_STEP, approximate_size);
        let cells_per_block = BLOCK_PAYLOAD / size_class;
        let possibly_better_size_class = (BLOCK_PAYLOAD / cells_per_block) & !(SIZE_STEP - 1);

        let original_wastage = BLOCK_PAYLOAD - cells_per_block * size_class;
        let new_wastage = (possibly_better_size_class - size_class) * cells_per_block;
        let better_size_class;
        if new_wastage > original_wastage {
            better_size_class = size_class;
        } else {
            better_size_class = possibly_better_size_class;
        }
        if &better_size_class == result.last().unwrap() {
            continue;
        }
        if better_size_class > LARGE_CUTOFF {
            break;
        }
        add(better_size_class, &mut result);
    }
    // Manually inject size classes for objects we know will be allocated in high volume.
    add(256, &mut result);
    // Sort and deduplicate.
    result.sort();
    result.dedup();

    result
});

pub fn build_size_class_table(
    table: &mut [usize],
    mut cons: impl FnMut(usize) -> usize,
    mut default_cons: impl FnMut(usize) -> usize,
) {
    let mut next_index = 0;
    for size_class in SIZE_CLASSES.iter() {
        let entry = cons(*size_class);
        let index = size_class_to_index(entry);
        for i in next_index..=index {
            table[i] = entry;
        }
        next_index = index + 1;
    }
    for i in next_index..NUM_SIZE_CLASSES {
        table[i] = default_cons(index_to_size_class(i));
    }
}

pub static SIZE_CLASS_FOR_SIZE_STEP: once_cell::sync::Lazy<[usize; NUM_SIZE_CLASSES]> =
    once_cell::sync::Lazy::new(|| {
        let mut arr = [0; NUM_SIZE_CLASSES];
        build_size_class_table(&mut arr, |sz| sz, |sz| sz);
        arr
    });
pub struct SegregatedSpace {
    directories: Vec<*mut BlockDirectory>,
}

impl SegregatedSpace {
    pub fn new() -> Self {
        Self {
            directories: Vec::new(),
        }
    }

    pub fn optimal_size_for(bytes: usize) -> usize {
        if bytes <= PRECISE_CUTOFF {
            return round_up_to_multiple_of(SIZE_STEP, bytes);
        } else if bytes <= LARGE_CUTOFF {
            SIZE_CLASS_FOR_SIZE_STEP[size_class_to_index(bytes)]
        } else {
            bytes
        }
    }

    pub fn shrink(&mut self) {
        for directory in self.directories.iter_mut() {
            unsafe { &mut **directory }.shrink();
        }
    }

    pub fn sweep(&mut self) {
        for directory in self.directories.iter_mut() {
            unsafe { &mut **directory }.sweep();
        }
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        unsafe {
            self.marks.take(); // "safe" drop for `marks`.
            super::aligned_free(self as *mut Self as *mut _);
        }
    }
}

pub struct SegregatedAllocator {
    allocator_for_size_step: [*mut BlockDirectory; NUM_SIZE_CLASSES],
    space: SegregatedSpace,
}

impl SegregatedAllocator {
    pub fn new() -> Self {
        Self {
            allocator_for_size_step: [core::ptr::null_mut(); NUM_SIZE_CLASSES],
            space: SegregatedSpace::new(),
        }
    }
    pub fn free_block(&mut self, block: *mut Block) {
        unsafe {
            let b = &mut *block;
            (&mut *b.directory).blocks.retain(|x| *x != block);
        }
    }

    pub fn allocate(&mut self, size: usize) -> Address {
        if let Some(alloc) = self.allocator_for(size) {
            alloc.allocate()
        } else {
            Address::null()
        }
    }

    pub fn allocator_for(&mut self, size: usize) -> Option<&mut BlockDirectory> {
        if size <= LARGE_CUTOFF {
            let result = self.allocator_for_size_step[size_class_to_index(size)];
            if result.is_null() {
                Some(self.allocator_for_slow(size))
            } else {
                Some(unsafe { &mut *result })
            }
        } else {
            None
        }
    }
    fn allocator_for_slow(&mut self, size: usize) -> &mut BlockDirectory {
        let index = size_class_to_index(size);
        let size_class = SIZE_CLASS_FOR_SIZE_STEP[index];
        let candidate = self.allocator_for_size_step[index];

        if candidate.is_null() == false {
            return unsafe { &mut *candidate };
        }
        let mut directory = Box::new(BlockDirectory::new(size_class));
        let raw: *mut BlockDirectory = &mut *directory;
        self.space.directories.push(raw);

        let mut index = size_class_to_index(size_class);
        loop {
            if SIZE_CLASS_FOR_SIZE_STEP[index] != size_class {
                break;
            }

            self.allocator_for_size_step[index] = raw;
            if index == 0 {
                break;
            }
            index -= 1;
        }
        unsafe { &mut *Box::into_raw(directory) }
    }
}
