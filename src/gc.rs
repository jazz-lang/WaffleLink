//! Mark & Sweep garbage collector.

use super::object::*;
use std::alloc::Layout;
use std::cmp::Ordering;
use std::sync::atomic::{spin_loop_hint, AtomicBool, AtomicUsize, Ordering as A};
use std::sync::Arc;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum BlockMark {
    NotMarked,
    Marked,
}
pub const K: usize = 1024;
pub const SIZE_CLASSES: usize = 5;

pub const SIZE_CLASS_SMALLEST: SizeClass = SizeClass(0);
pub const SIZE_SMALLEST: usize = 16;

pub const SIZE_CLASS_TINY: SizeClass = SizeClass(1);
pub const SIZE_TINY: usize = 32;

pub const SIZE_CLASS_SMALL: SizeClass = SizeClass(2);
pub const SIZE_SMALL: usize = 128;

pub const SIZE_CLASS_MEDIUM: SizeClass = SizeClass(3);
pub const SIZE_MEDIUM: usize = 2 * K;

pub const SIZE_CLASS_LARGE: SizeClass = SizeClass(4);
pub const SIZE_LARGE: usize = 8 * K;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SizeClass(usize);

pub const SIZES: [usize; SIZE_CLASSES] = [
    SIZE_SMALLEST,
    SIZE_TINY,
    SIZE_SMALL,
    SIZE_MEDIUM,
    SIZE_LARGE,
];

impl SizeClass {
    fn next_up(size: usize) -> Option<SizeClass> {
        assert!(size >= SIZE_SMALLEST);

        Some(if size <= SIZE_SMALLEST {
            SIZE_CLASS_SMALLEST
        } else if size <= SIZE_TINY {
            SIZE_CLASS_TINY
        } else if size <= SIZE_SMALL {
            SIZE_CLASS_SMALL
        } else if size <= SIZE_MEDIUM {
            SIZE_CLASS_MEDIUM
        } else if size <= SIZE_LARGE {
            SIZE_CLASS_LARGE
        } else {
            return None;
        })
    }

    fn next_down(size: usize) -> Option<SizeClass> {
        assert!(size >= SIZE_SMALLEST);

        Some(if size < SIZE_TINY {
            SIZE_CLASS_SMALLEST
        } else if size < SIZE_SMALL {
            SIZE_CLASS_TINY
        } else if size < SIZE_MEDIUM {
            SIZE_CLASS_SMALL
        } else if size < SIZE_LARGE {
            SIZE_CLASS_MEDIUM
        } else {
            return None;
        })
    }

    fn idx(self) -> usize {
        self.0
    }

    fn size(self) -> usize {
        SIZES[self.0]
    }
}

pub struct FreeList {
    classes: Vec<FreeListClass>,
}

impl FreeList {
    pub fn new() -> FreeList {
        let mut classes = Vec::with_capacity(SIZE_CLASSES);

        for _ in 0..SIZE_CLASSES {
            classes.push(FreeListClass::new());
        }

        FreeList { classes }
    }

    pub fn add(&mut self, addr: Address, size: usize) -> bool {
        if size < SIZE_SMALLEST {
            return false;
        }

        debug_assert!(size >= SIZE_SMALLEST);
        let szclass = match SizeClass::next_down(size) {
            Some(cls) => cls,
            _ => return false,
        };

        let free_class = &mut self.classes[szclass.idx()];
        free_class.head = FreeSpace(addr);
        true
    }

    pub fn alloc(&mut self, size: usize) -> Option<FreeSpace> {
        let szclass = SizeClass::next_up(size)?.idx();

        for class in szclass..5 {
            let result = self.classes[class].first();

            if result.is_non_null() {
                assert!(result.size() >= size);
                return Some(result);
            }
        }

        None // can't allocate huge object or no memory is found
    }
}

pub struct FreeListClass {
    head: FreeSpace,
}

impl FreeListClass {
    fn new() -> FreeListClass {
        FreeListClass {
            head: FreeSpace::null(),
        }
    }

    fn add(&mut self, addr: FreeSpace) {
        addr.set_next(self.head);
        self.head = addr;
    }

    fn first(&mut self) -> FreeSpace {
        if self.head.is_non_null() {
            let ret = self.head;
            self.head = ret.next();
            ret
        } else {
            FreeSpace::null()
        }
    }

    fn find(&mut self, minimum_size: usize) -> FreeSpace {
        let mut curr = self.head;
        let mut prev = FreeSpace::null();

        while curr.is_non_null() {
            if curr.size() >= minimum_size {
                if prev.is_null() {
                    self.head = curr.next();
                } else {
                    prev.set_next(curr.next());
                }

                return curr;
            }

            prev = curr;
            curr = curr.next();
        }

        FreeSpace::null()
    }
}
#[derive(Copy, Clone)]
pub struct FreeSpace(Address);

impl FreeSpace {
    #[inline(always)]
    pub fn null() -> FreeSpace {
        FreeSpace(Address::null())
    }

    #[inline(always)]
    pub fn is_null(self) -> bool {
        self.addr().is_null()
    }

    #[inline(always)]
    pub fn is_non_null(self) -> bool {
        self.addr().is_non_null()
    }

    #[inline(always)]
    pub fn addr(self) -> Address {
        self.0
    }

    #[inline(always)]
    pub fn next(self) -> FreeSpace {
        assert!(self.is_non_null());
        let next = unsafe { *self.addr().add_ptr(1).to_mut_ptr::<Address>() };
        FreeSpace(next)
    }

    #[inline(always)]
    pub fn set_next(&self, next: FreeSpace) {
        assert!(self.is_non_null());
        unsafe { *self.addr().add_ptr(1).to_mut_ptr::<Address>() = next.addr() }
    }

    #[inline(always)]
    pub fn size(self) -> usize {
        let obj = self.addr().to_mut_ptr::<FreeCell>();
        unsafe { (&*obj).size as usize }
    }
}

/// This *must* be <= 16 bytes
pub struct FreeCell {
    size: usize,
    next: *mut FreeCell,
}

pub const BLOCK_SIZE: usize = 32 * 1024;
pub const BLOCK_MASK: usize = !(BLOCK_SIZE - 1);

fn block_layout() -> Layout {
    Layout::from_size_align(BLOCK_SIZE, BLOCK_SIZE).unwrap()
}

pub struct BlockHeader {
    pub block: *mut Block,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Address(usize);

impl Address {
    #[inline(always)]
    pub fn from(val: usize) -> Address {
        Address(val)
    }

    #[inline(always)]
    pub fn region_start(self, size: usize) -> Region {
        Region::new(self, self.offset(size))
    }

    #[inline(always)]
    pub fn offset_from(self, base: Address) -> usize {
        debug_assert!(self >= base);

        self.to_usize() - base.to_usize()
    }

    #[inline(always)]
    pub fn offset(self, offset: usize) -> Address {
        Address(self.0 + offset)
    }

    #[inline(always)]
    pub fn sub(self, offset: usize) -> Address {
        Address(self.0 - offset)
    }

    #[inline(always)]
    pub fn add_ptr(self, words: usize) -> Address {
        Address(self.0 + words * std::mem::size_of::<usize>())
    }

    #[inline(always)]
    pub fn sub_ptr(self, words: usize) -> Address {
        Address(self.0 - words * std::mem::size_of::<usize>())
    }

    #[inline(always)]
    pub fn to_usize(self) -> usize {
        self.0
    }

    #[inline(always)]
    pub fn from_ptr<T>(ptr: *const T) -> Address {
        Address(ptr as usize)
    }

    #[inline(always)]
    pub fn to_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    #[inline(always)]
    pub fn to_mut_ptr<T>(&self) -> *mut T {
        self.0 as *const T as *mut T
    }

    #[inline(always)]
    pub fn null() -> Address {
        Address(0)
    }

    #[inline(always)]
    pub fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub fn is_non_null(self) -> bool {
        self.0 != 0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:x}", self.to_usize())
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:x}", self.to_usize())
    }
}

impl PartialOrd for Address {
    fn partial_cmp(&self, other: &Address) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Address {
    fn cmp(&self, other: &Address) -> Ordering {
        self.to_usize().cmp(&other.to_usize())
    }
}

impl From<usize> for Address {
    fn from(val: usize) -> Address {
        Address(val)
    }
}

#[derive(Copy, Clone)]
pub struct Region {
    pub start: Address,
    pub end: Address,
}

impl Region {
    pub fn new(start: Address, end: Address) -> Region {
        debug_assert!(start <= end);

        Region { start, end }
    }

    #[inline(always)]
    pub fn contains(&self, addr: Address) -> bool {
        self.start <= addr && addr < self.end
    }

    #[inline(always)]
    pub fn valid_top(&self, addr: Address) -> bool {
        self.start <= addr && addr <= self.end
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.end.to_usize() - self.start.to_usize()
    }

    #[inline(always)]
    pub fn empty(&self) -> bool {
        self.start == self.end
    }

    #[inline(always)]
    pub fn disjunct(&self, other: &Region) -> bool {
        self.end <= other.start || self.start >= other.end
    }

    #[inline(always)]
    pub fn overlaps(&self, other: &Region) -> bool {
        !self.disjunct(other)
    }

    #[inline(always)]
    pub fn fully_contains(&self, other: &Region) -> bool {
        self.contains(other.start) && self.valid_top(other.end)
    }
}

impl Default for Region {
    fn default() -> Region {
        Region {
            start: Address::null(),
            end: Address::null(),
        }
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

struct FormattedSize {
    size: usize,
}
use std::fmt;
impl fmt::Display for FormattedSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ksize = (self.size as f64) / 1024f64;

        if ksize < 1f64 {
            return write!(f, "{}B", self.size);
        }

        let msize = ksize / 1024f64;

        if msize < 1f64 {
            return write!(f, "{:.1}K", ksize);
        }

        let gsize = msize / 1024f64;

        if gsize < 1f64 {
            write!(f, "{:.1}M", msize)
        } else {
            write!(f, "{:.1}G", gsize)
        }
    }
}

fn formatted_size(size: usize) -> FormattedSize {
    FormattedSize { size }
}

pub struct Block {
    pub needs_sweep: bool,
    pub mark: BlockMark,
    pub free_list: FreeList,
    pub data: Address,
    pub start: Address,
    pub end: Address,
    pub cursor: Address,
}

pub enum AllocResult {
    Bump(Address),
    FreeCell(FreeSpace),
    HugeObject,
    NoMemFound,
    NoBlocks,
}

pub trait Bool {
    const RESULT: bool;
}

pub struct True;
impl Bool for True {
    const RESULT: bool = true;
}
pub struct False;
impl Bool for False {
    const RESULT: bool = false;
}

impl Block {
    pub unsafe fn header(&self) -> &mut BlockHeader {
        &mut *self.data.to_mut_ptr::<BlockHeader>()
    }
    pub unsafe fn sweep<SweepToFreeList: Bool>(&mut self) -> bool {
        if self.mark == BlockMark::NotMarked || !SweepToFreeList::RESULT {
            let mut scan = self.start;
            let end = self.end;
            while scan < end {
                let addr = scan;
                let mem: *mut Cell = addr.to_mut_ptr::<Cell>();
                let size = {
                    let sz = (&*mem).size();
                    (&mut *addr.to_mut_ptr::<Cell>()).finalize();
                    sz
                };
                scan = scan.offset(size);
            }
            return true;
        }
        self.needs_sweep = false;
        let mut scan = self.start;
        let end = self.end;
        let mut garbage_start = Address::null();
        let mut all_free = true;
        let mut freelist = FreeList::new();
        let mut add_freelist = |start: Address, end: Address| {
            if start.is_null() {
                return;
            }
            let size = end.offset_from(start);
            freelist.add(start, size);
        };
        while scan < end {
            let addr = scan;
            let mem: *mut Cell = addr.to_mut_ptr::<Cell>();
            let size = if (&*mem).mark {
                add_freelist(garbage_start, scan);
                (&mut *mem).mark = false;
                garbage_start = Address::null();
                all_free = false;
                (&*mem).size()
            } else {
                if garbage_start.is_null() {
                    garbage_start = scan;
                }
                let sz = (&*mem).size();
                (&mut *addr.to_mut_ptr::<Cell>()).finalize();
                sz
            };
            scan = scan.offset(size);
        }
        add_freelist(garbage_start, end);
        self.mark = BlockMark::NotMarked;
        drop(std::mem::replace(&mut self.free_list, freelist));
        if all_free {
            self.cursor = self.start;
        }
        all_free
    }
    fn take_free(&mut self, size: usize) -> Option<FreeSpace> {
        self.free_list.alloc(size)
    }

    pub fn allocate_memory(&mut self, size: usize) -> AllocResult {
        if size <= 8 * K {
            if self.cursor.offset(size) < self.end {
                let prev = self.cursor;
                self.cursor = self.cursor.offset(size);
                return AllocResult::Bump(prev);
            } else if let Some(space) = self.take_free(size) {
                return AllocResult::FreeCell(space);
            } else {
                return AllocResult::NoMemFound;
            }
        } else {
            // Huge objects go to different space
            return AllocResult::HugeObject;
        }
    }

    pub unsafe fn from_pointer<'a, T>(ptr: *const T) -> Option<&'a mut Self> {
        let ptr = ptr as usize;
        let res = ptr & BLOCK_MASK;
        if res == 0 {
            return None;
        }
        let ptr = res as *mut BlockHeader;
        Some(&mut *(&*ptr).block)
    }
}

use std::collections::LinkedList;
use std::ptr::NonNull;
pub struct LocalAllocator {
    current: Option<NonNull<Block>>,
}

impl LocalAllocator {
    pub unsafe fn try_allocate_memory(&mut self, size: usize) -> AllocResult {
        match self.current {
            Some(ref mut b) => b.as_mut().allocate_memory(size),
            None => AllocResult::NoBlocks,
        }
    }

    pub unsafe fn allocate_memory(&mut self, size: usize) -> Address {
        loop {
            match self.try_allocate_memory(size) {
                AllocResult::Bump(addr) => return addr,
                AllocResult::FreeCell(space) => return space.addr(),
                _ => {
                    self.init_block(true);
                    continue;
                }
            }
        }
    }

    unsafe fn init_block(&mut self, lock: bool) {
        self.current = Some(GLOBAL_ALLOC.next_block(lock));
    }
}
use parking_lot::{lock_api::RawMutex, RawMutex as Lock};
use std::cell::UnsafeCell;
pub struct GlobalAllocator {
    global_lock: Lock,
    all_blocks: UnsafeCell<Vec<Box<Block>>>,
    free_list: UnsafeCell<LinkedList<NonNull<Block>>>,
    recyclable: UnsafeCell<LinkedList<NonNull<Block>>>,
    stopping_world: AtomicBool,
    threads: UnsafeCell<Vec<Arc<GcThread>>>,
    allocated_bytes: AtomicUsize,
    threshold: AtomicUsize,
}

impl GlobalAllocator {
    pub fn new() -> Self {
        Self {
            global_lock: Lock::INIT,
            all_blocks: UnsafeCell::new(vec![]),
            free_list: UnsafeCell::new(LinkedList::new()),
            recyclable: UnsafeCell::new(LinkedList::new()),
            stopping_world: AtomicBool::new(false),
            threads: UnsafeCell::new(Vec::new()),
            allocated_bytes: AtomicUsize::new(0),
            threshold: AtomicUsize::new(16 * 1024),
        }
    }
    pub unsafe fn next_block(&self, lock: bool) -> NonNull<Block> {
        self.global_lock(lock);
        if let Some(b) = (&mut *self.free_list.get()).pop_front() {
            if lock {
                self.global_lock(false);
            }
            return b;
        }
        if let Some(b) = (&mut *self.recyclable.get()).pop_front() {
            if (&*b.as_ptr()).needs_sweep {
                (&mut *b.as_ptr()).sweep::<True>();
            }
            if lock {
                self.global_lock(false);
            }
            return b;
        }

        unimplemented!();
        // TODO: Grow heap.
        if lock {
            self.global_lock(false);
        };
    }

    pub unsafe fn global_lock(&self, b: bool) {
        if b {
            GC_THREAD.with(|x| x.borrow().blocking.fetch_add(1, A::Relaxed));
            self.global_lock.lock();
        } else {
            GC_THREAD.with(|x| x.borrow().blocking.fetch_sub(1, A::Relaxed));
            self.global_lock.unlock();
        }
    }
    /// Trigger minor GC.
    pub unsafe fn gc_minor(&self) {
        use super::object::*;
        self.gc_stop_world(true);
        // TODO: Mark roots
        let mut mark_stack = Vec::<*const Cell>::new();
        while let Some(elem) = mark_stack.pop() {
            let object_ref = &mut *(elem as *mut Cell);

            object_ref.visit(&mut |elem| {
                let object_ref = &mut *(elem as *mut Object);
                if object_ref.mark == false {
                    object_ref.mark = true;
                    let block = Block::from_pointer(object_ref).unwrap();
                    block.mark = BlockMark::Marked;
                    mark_stack.push(elem);
                }
            });
        }
        let threads_count = (&*self.threads.get()).len();
        let mut free = 0;
        retain_mut(&mut *self.all_blocks.get(), |block| {
            // Block is marked, sweep value to block freelist and if all values is free
            // push it to global allocator block freelist.

            if block.mark == BlockMark::Marked {
                block.needs_sweep = true;
                block.mark = BlockMark::NotMarked;
                // Block is marked, but not all values is freed so just push block to recyclable list.
                (&mut *self.recyclable.get()).push_back(NonNull::new_unchecked(&mut **block));
                true
            } else {
                block.needs_sweep = false;
                block.mark = BlockMark::NotMarked;
                free += 1;
                // too much free blocks, free to OS.
                if free >= (threads_count as f64 * 4.5) as usize {
                    false
                } else {
                    (&mut *self.free_list.get()).push_back(NonNull::new_unchecked(&mut **block));
                    true
                }
            }
        });

        for thread in (&*self.threads.get()).iter() {
            (&mut *thread.local.get()).init_block(false);
        }
        self.gc_stop_world(false);
    }
    // Trigger major GC.
    pub unsafe fn gc_major(&self) {
        use super::object::*;
        self.gc_stop_world(true);
        // TODO: Mark roots
        let mut mark_stack = Vec::<*const Cell>::new();
        while let Some(elem) = mark_stack.pop() {
            let object_ref = &mut *(elem as *mut Cell);

            object_ref.visit(&mut |elem| {
                let object_ref = &mut *(elem as *mut Object);
                if object_ref.mark == false {
                    object_ref.mark = true;
                    let block = Block::from_pointer(object_ref).unwrap();
                    block.mark = BlockMark::Marked;
                    mark_stack.push(elem);
                }
            });
        }
        let threads_count = (&*self.threads.get()).len();
        let mut free = 0;
        // Sweep heap blocks, this will do these things:
        retain_mut(&mut *self.all_blocks.get(), |block| {
            // Block is marked, sweep value to block freelist and if all values is free
            // push it to global allocator block freelist.

            if block.sweep::<True>() {
                free += 1;
                // too much free blocks, free to OS.
                if free >= (threads_count as f64 * 2.5) as usize {
                    false
                } else {
                    (&mut *self.free_list.get()).push_back(NonNull::new_unchecked(&mut **block));
                    true
                }
            } else {
                // Block is marked, but not all values is freed so just push block to recyclable list.
                (&mut *self.recyclable.get()).push_back(NonNull::new_unchecked(&mut **block));
                true
            }
        });

        for thread in (&*self.threads.get()).iter() {
            (&mut *thread.local.get()).init_block(false);
        }
        self.gc_stop_world(false);
    }

    pub unsafe fn gc_stop_world(&self, b: bool) {
        if !b {
            self.stopping_world.store(false, A::Relaxed);
            self.global_lock(false);
        } else {
            self.global_lock(true);
            self.stopping_world.store(true, A::Relaxed);
            for thread in &*self.threads.get() {
                while thread.blocking.load(A::Acquire) == 0 {
                    spin_loop_hint();
                }
            }
        }
    }
}
unsafe impl Send for GlobalAllocator {}
unsafe impl Sync for GlobalAllocator {}
lazy_static::lazy_static! {
    static ref GLOBAL_ALLOC: GlobalAllocator = GlobalAllocator::new();
}
pub struct GcThread {
    id: usize,
    blocking: AtomicUsize,
    local: UnsafeCell<LocalAllocator>,
}

use std::cell::RefCell;

thread_local! {
    pub static GC_THREAD: RefCell<Arc<GcThread>> = RefCell::new(Arc::new(GcThread {
        id: 0,
        blocking: AtomicUsize::new(0),
        local: UnsafeCell::new(LocalAllocator {
            current: None
        })
    }))
}

pub fn retain_mut<T>(v: &mut Vec<T>, mut f: impl FnMut(&mut T) -> bool) {
    for i in (0..v.len()).rev() {
        // Process the item, determine if it should be removed
        let should_remove = {
            // Everyone take some damage! Remove the dead!
            let elem = &mut v[i];
            f(elem)
        };
        if should_remove {
            // Swap this item with the end of the array, and then
            // pop it off. This "scrambles" the array, but that's
            // ok because we don't care about order. Also, the only
            // elements that are scrambled are the ones we've already
            // seen, so this won't cause us to accidentally skip or
            // reprocess an enemy. Further, the fact that the `len` of
            // the array is decreased by this op doesn't matter to us,
            // because we're about to go to a smaller index.
            v.swap_remove(i);
        }
    }
}
