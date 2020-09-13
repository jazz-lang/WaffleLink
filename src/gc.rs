//! General interface to GC implementations.

pub mod bitmap;
pub mod block;
pub mod lazysweep;
//#[cfg(feature = "pmarking")]
//pub mod pmarking;
//pub mod cmarking;
pub mod object;
pub mod pagealloc;

pub mod precise_allocation;
use object::*;
/// GC didn't seen this object.
pub const GC_WHITE: u8 = 0;
/// Object fields was visited
pub const GC_BLACK: u8 = 2;
/// Object is in graylist
pub const GC_GRAY: u8 = 1;
/// Old gen object
pub const GC_BLUE: u8 = 3;
pub const GC_NEW: u8 = 0;
pub const GC_OLD: u8 = 1;
pub const GC_NONE: u8 = 3;
pub const GC_OLD_REMEMBERED: u8 = 2;

pub const GC_VERBOSE_LOG: bool = true;
pub const GC_LOG: bool = true;
pub const GC_LOG_TIMINGS: bool = true;
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Address(usize);

impl Address {
    #[inline(always)]
    pub fn from(val: usize) -> Address {
        Address(val)
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
        Address(self.0 + words * core::mem::size_of::<usize>())
    }

    #[inline(always)]
    pub fn sub_ptr(self, words: usize) -> Address {
        Address(self.0 - words * core::mem::size_of::<usize>())
    }

    #[inline(always)]
    pub fn to_mut_obj(self) -> &'static mut GcBox<()> {
        unsafe { &mut *self.to_mut_ptr::<_>() }
    }

    #[inline(always)]
    pub fn to_obj(self) -> &'static GcBox<()> {
        unsafe { &*self.to_mut_ptr::<_>() }
    }

    #[inline(always)]
    pub const fn to_usize(self) -> usize {
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
use std::cmp::Ordering;
use std::fmt;
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

pub const fn round_up_to_multiple_of(divisor: usize, x: usize) -> usize {
    (x + (divisor - 1)) & !(divisor - 1)
}

pub trait GarbageCollector {
    fn minor(&mut self) {
        self.major();
    }

    fn major(&mut self) {
        self.full();
    }

    fn full(&mut self);

    fn allocate(&mut self, size: usize) -> Address;

    fn new_local_scope(&mut self) -> LocalScope;

    /// Perform write barrier operation.
    #[allow(dead_code)]
    fn write_barrier(&mut self, object: *mut GcBox<()>, field: *mut GcBox<()>) {}
    // TODO: When we will have JIT this function should emit proper write barrier.
    fn emit_write_barrier(&self) {}
}

pub struct Heap {
    gc: Box<dyn GarbageCollector>,
}

impl Heap {
    pub fn new<T: GarbageCollector + 'static>(gc: T) -> Self {
        Self { gc: Box::new(gc) }
    }

    pub fn lazysweep() -> Self {
        Self::new(lazysweep::LazySweepGC::new())
    }

    pub fn new_local_scope(&mut self) -> LocalScope {
        self.gc.new_local_scope()
    }

    pub fn allocate<T: GcObject>(&mut self, value: T) -> Handle<T> {
        unsafe { gc_alloc_handle(&mut *self.gc, value) }
    }

    pub fn major(&mut self) {
        self.gc.major();
    }

    pub fn minor(&mut self) {
        self.gc.minor();
    }

    pub fn full(&mut self) {
        self.gc.full();
    }

    pub fn write_barrier<T: GcObject, U: GcObject>(&mut self, object: Handle<T>, field: Handle<U>) {
        self.gc
            .write_barrier(object.gc_ptr().cast(), field.gc_ptr().cast());
    }
}

/// Use `dyn GarbageCollector` to allocate `Handle<T>` properly.
pub unsafe fn gc_alloc_handle<T: GcObject>(gc: &mut dyn GarbageCollector, value: T) -> Handle<T> {
    let raw = gc.allocate(value.size() + core::mem::size_of::<Header>());

    raw.to_mut_ptr::<GcBox<T>>().write(GcBox {
        header: Header {
            cell_state: 0,
            vtable: std::mem::transmute::<_, TraitObject>(&value as &dyn GcObject).vtable,
        },
        value,
    });

    Handle {
        ptr: core::ptr::NonNull::new_unchecked(raw.to_mut_ptr()),
    }
}
