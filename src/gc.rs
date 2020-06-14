//! Mark & Sweep garbage collector.

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
pub struct GlobalAllocator {
    global_lock: Lock,
    stopping_world: AtomicBool,
    threads: UnsafeCell<Vec<Arc<GcThread>>>,
    allocated_bytes: AtomicUsize,
    threshold: AtomicUsize,
}

impl GlobalAllocator {
    pub fn new() -> Self {
        Self {
            global_lock: Lock::INIT,
            stopping_world: AtomicBool::new(false),
            threads: UnsafeCell::new(Vec::new()),
            allocated_bytes: AtomicUsize::new(0),
            threshold: AtomicUsize::new(16 * 1024),
        }
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
}

use std::cell::RefCell;

thread_local! {
    pub static GC_THREAD: RefCell<Arc<GcThread>> = RefCell::new(Arc::new(GcThread {
        id: 0,
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
