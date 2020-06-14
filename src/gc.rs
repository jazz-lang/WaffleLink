//! Mark & Sweep garbage collector.
pub mod block;
use super::object::*;
use std::alloc::Layout;
use std::cmp::Ordering;
use std::sync::atomic::{spin_loop_hint, AtomicBool, AtomicUsize, Ordering as A};
use std::sync::Arc;

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
use parking_lot::{lock_api::RawMutex, RawMutex as Lock};
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
pub struct GlobalAllocator {
    global_lock: Lock,
    stopping_world: AtomicBool,
    all_blocks: UnsafeCell<Vec<Box<block::Block>>>,
    threads: UnsafeCell<Vec<Arc<GcThread>>>,
    allocated_bytes: AtomicUsize,
    threshold: AtomicUsize,
}

static THREAD_ID: AtomicUsize = AtomicUsize::new(1);

impl GlobalAllocator {
    pub fn new() -> Self {
        Self {
            global_lock: Lock::INIT,
            stopping_world: AtomicBool::new(false),
            threads: UnsafeCell::new(Vec::new()),
            all_blocks: UnsafeCell::new(vec![]),
            allocated_bytes: AtomicUsize::new(0),
            threshold: AtomicUsize::new(16 * 1024),
        }
    }
    pub unsafe fn global_lock(&self, b: bool) {
        if b {
            Self::gc_save_ctx(Self::current_thread(), &b);
            GC_THREAD.with(|x| x.borrow().blocking.fetch_add(1, A::Relaxed));
            
            self.global_lock.lock();
        } else {
            GC_THREAD.with(|x| x.borrow().blocking.fetch_sub(1, A::Relaxed));
            self.global_lock.unlock();
        }
    }
    pub fn blocking(b: bool) {
        let t = Self::current_thread();
        if t.id == 0 {
            return;
        }
        unsafe {
            if b {
                if t.blocking.fetch_add(1,A::AcqRel) == 0 {
                    Self::gc_save_ctx(t.clone(), &b);
                }
            } else if t.blocking.load(A::Relaxed) == 0 {
                panic!("Unblocked thread");
            } else {
                t.blocking.fetch_sub(1,A::AcqRel);
                if t.blocking.fetch_sub(1,A::AcqRel) == 1 && GLOBAL_ALLOC.stopping_world.load(A::Acquire) {
                    GLOBAL_ALLOC.global_lock(true);
                    GLOBAL_ALLOC.global_lock(false);
                }
            }
        
        }
    }

    pub fn register_thread<STACK>(&self,stack_top: *const STACK) {
        let stack_top = stack_top as *mut u8;
        let thread = GcThread {
            id: THREAD_ID.fetch_add(1,A::AcqRel),
            stack_start: UnsafeCell::new(stack_top),
            stack_cur: UnsafeCell::new(std::ptr::null_mut()),
            extra_stack_size: UnsafeCell::new(0),
            extra_stack_data: UnsafeCell::new(0),
            regs: MaybeUninit::uninit(),
            blocking: AtomicUsize::new(0),
        };
        GC_THREAD.with(|th| unsafe {
            *th.borrow_mut() = Arc::new(thread);
            self.global_lock(true);
            (&mut *self.threads.get()).push(th.borrow().clone());
            self.global_lock(false);
        });
    }
    pub unsafe fn gc_save_ctx<T>(t: Arc<GcThread>, prev_stack: *const T) {
        let stack_cur = &t as *const _ as *mut u8;
        setjmp::setjmp(t.regs.as_ptr() as *mut _);
        *(&mut *t.stack_cur.get()) = stack_cur;
        // LLVM might push/pop some callee registers in call to gc_save_ctx (or before)
        // which might hold a GC value, let's capture them immediately in extra per thread data.
        let size = (prev_stack as usize - stack_cur as usize) / std::mem::size_of::<usize>();
        *(&mut *t.extra_stack_size.get()) = size;
        std::ptr::copy_nonoverlapping(
            prev_stack as *const u8,
            (*t.extra_stack_data.get()) as *mut u8,
            size * std::mem::size_of::<usize>(),
        );
    }
    pub fn current_thread() -> Arc<GcThread> {
        GC_THREAD.with(|th| th.borrow().clone())
    }
    /// Check is pointer in GC heap or no, if no this function returns false otherwise true is
    /// returned.
    unsafe fn ptr_in_heap(&self,ptr: *mut u8) -> bool {
        use block::*;
        let block = Block::from_pointer(ptr);
        if block.is_none() { // Null block pointer == ptr is not from heap.
            return false;
        }
        let ptr = Address::from_ptr(ptr);
        let block = block.unwrap();
        // Search for block in all heap blocks & check that pointer is in bounds of allocated
        // block.
        (&*self.all_blocks.get()).iter().find(|x| ((&***x) as *const Block) == block as *const Block).is_some() 
            && ptr >= block.start && ptr <= block.limit

    }

    /// Conservatively mark thread stacks.
    unsafe fn mark_thread_stacks(&self,mark_stack: &mut Vec<WaffleCellPointer>) {
        todo!();
    }

    /// Waffle GC is conservative on stack and precise on heap, this function scans threads stack 
    /// for GC roots. Can it identify some random integer as pointer? Sure it can but on x64 this
    /// is very rare and basically impossible. 
    ///
    /// ## Safety 
    /// This is function is unsafe because we use UnsafeCell internally to modify contents behind 
    /// Arc pointer and do two transmutes for fast convertation pointer<->value, but in other parts
    /// it is fully safe.
    unsafe fn mark_thread(&self,mark_stack: &mut Vec<WaffleCellPointer>,th: &Arc<GcThread>) {
        let mut start = *th.stack_start.get();
        let mut end = *th.stack_cur.get();
        if start > end {
            std::mem::swap(&mut start, &mut end);
        }

        while start < end {
            let scan = *(start as *mut *mut u8);
            if scan.is_null() == false || std::mem::transmute::<_,super::value::Value>(scan).is_cell() != false {
                if self.ptr_in_heap(scan) {
                    mark_stack.push(std::mem::transmute(scan));
                }
            }
            start = start.offset(std::mem::size_of::<usize>() as _);
        }
    }
    pub unsafe fn gc_stop_world(&self, b: bool) {
        if !b {
            self.stopping_world.store(false, A::SeqCst);
            self.global_lock(false);
        } else {
            self.global_lock(true);
            self.stopping_world.store(true, A::SeqCst);
            for thread in &*self.threads.get() {
                while thread.blocking.load(A::Acquire) == 0 {
                    spin_loop_hint();
                }
            }
        }
        if b {
            Self::gc_save_ctx(Self::current_thread(), &b);
        }
    }

    pub unsafe fn safepoint(&self) {
        if self.stopping_world.load(A::Acquire) {
            GC_THREAD.with(|x| x.borrow().blocking.fetch_add(1, A::Release));
            while self.stopping_world.load(A::Acquire) {
                spin_loop_hint();
            }
            GC_THREAD.with(|x| x.borrow().blocking.fetch_sub(1, A::Relaxed));
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
    regs: MaybeUninit<setjmp::jmp_buf>,
    stack_cur: UnsafeCell<*mut u8>,
    stack_start: UnsafeCell<*mut u8>,
    extra_stack_size: UnsafeCell<usize>,
    extra_stack_data: UnsafeCell<usize>,
    blocking: AtomicUsize,
}

use std::cell::RefCell;

thread_local! {
    pub static GC_THREAD: RefCell<Arc<GcThread>> = RefCell::new(Arc::new(GcThread {
        id: 0,
        stack_start: UnsafeCell::new(std::ptr::null_mut()),
        stack_cur: UnsafeCell::new(std::ptr::null_mut()),
        extra_stack_size: UnsafeCell::new(0),
        extra_stack_data: UnsafeCell::new(0),
        regs: MaybeUninit::uninit(),
        blocking: AtomicUsize::new(0),
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
