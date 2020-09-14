//! General interface to GC implementations.

pub mod bitmap;
/// Heap block
pub mod block;
/// LazySweepGC
pub mod lazysweep;
//#[cfg(feature = "pmarking")]
//pub mod pmarking;
//pub mod cmarking;
/// Object repr
pub mod object;
/// Page allocator
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
/// Young object
pub const GC_NEW: u8 = 0;
/// Old object
pub const GC_OLD: u8 = 1;
/// None object
pub const GC_NONE: u8 = 3;
/// Remembered old object
pub const GC_OLD_REMEMBERED: u8 = 2;
/// Enable verbose logging
pub const GC_VERBOSE_LOG: bool = true;
/// Enable logging
pub const GC_LOG: bool = true;
/// Enable time logging
pub const GC_LOG_TIMINGS: bool = true;

/// Wrapper around usize for easy pointer math.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Address(usize);

impl Address {
    /// Construct Self from usize
    #[inline(always)]
    pub fn from(val: usize) -> Address {
        Address(val)
    }
    /// Offset from `base`
    #[inline(always)]
    pub fn offset_from(self, base: Address) -> usize {
        debug_assert!(self >= base);

        self.to_usize() - base.to_usize()
    }
    /// Return self + offset
    #[inline(always)]
    pub fn offset(self, offset: usize) -> Address {
        Address(self.0 + offset)
    }
    /// Return self - offset
    #[inline(always)]
    pub fn sub(self, offset: usize) -> Address {
        Address(self.0 - offset)
    }
    /// Add pointer to self.
    #[inline(always)]
    pub fn add_ptr(self, words: usize) -> Address {
        Address(self.0 + words * core::mem::size_of::<usize>())
    }
    /// Sub pointer to self
    #[inline(always)]
    pub fn sub_ptr(self, words: usize) -> Address {
        Address(self.0 - words * core::mem::size_of::<usize>())
    }
    /// Convert address to object pointer.
    #[inline(always)]
    pub fn to_mut_obj(self) -> &'static mut GcBox<()> {
        unsafe { &mut *self.to_mut_ptr::<_>() }
    }
    /// Convert address to immutable object pointer.
    #[inline(always)]
    pub fn to_obj(self) -> &'static GcBox<()> {
        unsafe { &*self.to_mut_ptr::<_>() }
    }
    /// Convert pointer to usize
    #[inline(always)]
    pub const fn to_usize(self) -> usize {
        self.0
    }
    /// Construct from pointer
    #[inline(always)]
    pub fn from_ptr<T>(ptr: *const T) -> Address {
        Address(ptr as usize)
    }
    /// Convert to *const T
    #[inline(always)]
    pub fn to_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }
    /// Convert to *mut T
    #[inline(always)]
    pub fn to_mut_ptr<T>(&self) -> *mut T {
        self.0 as *const T as *mut T
    }
    /// Create null pointer
    #[inline(always)]
    pub fn null() -> Address {
        Address(0)
    }
    /// Check if self is null
    #[inline(always)]
    pub fn is_null(self) -> bool {
        self.0 == 0
    }
    /// Check if self is non null
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
/// Rounds up `x` to multiple of `divisor`
pub const fn round_up_to_multiple_of(divisor: usize, x: usize) -> usize {
    (x + (divisor - 1)) & !(divisor - 1)
}

/// Trait that allows us to have multiple GC supported in runtime.
/// When you instantiate Isolate you can pass your own implementation
/// of GC or use existing one.
///
///
/// # GC Implementation
/// There is only a few restrictions on how your GC should behave. You might even just
/// use BDWGC.
///  
/// - GC must invoke `drop_in_place` on allocated `GcBox<T>` when object is dead.
/// - GC should take care of synchronization on it's own.
///     Each GC operation that might stop the world or allocate should be guarded
///     by mutex since one Isolate might try to allocate object in another Isolate.
///
///
/// # GC overview
/// ## Lazy sweep GC
/// Simple mark&sweep garbage collector. This GC is incremental in terms
/// that sweep happens when you try to allocate from block that wasn't
/// sweeped yet. We use `std::alloc::alloc` for large allocations and they're
/// sweeped immediately after marking because we cannot make them sweep
/// lazily as we do not have freelist for large objects.
/// ## Lazy GC
/// Lazy GC is almost the same as Lazy Sweep GC but has incremental marking support.
/// ## Mark&Sweep GC
/// Slow and stupid implementation of mark&sweep. Marking and sweep is done in single cycle
/// and might take a lot of time so this GC is not recommended if you need
/// high latency.
///
///
pub trait GarbageCollector {
    /// Perform minor GC.
    /// For incremental GC this means do one GC step and for
    /// generational this means collect only young objects.
    fn minor(&mut self) {
        self.major();
    }
    /// Perform major GC.
    ///
    /// For generational GC this means collect old objects and
    /// for incremental GC this means full STW and collect entire
    /// heap.
    fn major(&mut self) {
        self.full();
    }
    /// Full GC.
    ///
    /// Collect all generations if GC is generational.
    fn full(&mut self);
    /// Allocate `size` bytes and collect garbage if necessary.
    fn allocate(&mut self, size: usize) -> Address;
    /// Allocate without GC.
    fn allocate_no_gc(&mut self, size: usize) -> Address;

    /// Create new local scope.
    fn new_local_scope(&mut self) -> LocalScope;

    /// Perform write barrier operation.
    #[allow(dead_code)]
    fn write_barrier(&mut self, object: *mut GcBox<()>, field: *mut GcBox<()>) {}
    /// Emit write barrier code.
    // TODO: When we will have JIT API this function should have proper signature
    fn emit_write_barrier(&self) {}
    /// Disable GC.
    fn defer_gc(&mut self);
    /// Enable GC.
    fn undefer_gc(&mut self);
    /// Allow GC to collect roots from isolate.
    fn set_isolate(&mut self, isolate: *mut crate::isolate::Isolate);
}
/// Heap is "wrapper" for `GarbageCollector`
pub struct Heap {
    gc: Box<dyn GarbageCollector>,
}

impl Heap {
    /// Create new GC instance `T`
    pub fn new<T: GarbageCollector + 'static>(gc: T) -> Self {
        Self { gc: Box::new(gc) }
    }
    /// Create new Lazy sweep GC instance.
    pub fn lazysweep() -> Self {
        Self::new(lazysweep::LazySweepGC::new())
    }
    /// Create new local scope.
    pub fn new_local_scope(&mut self) -> LocalScope {
        self.gc.new_local_scope()
    }
    /// Allocate `T` on GC heap.
    pub fn allocate<T: GcObject>(&mut self, value: T) -> Handle<T> {
        unsafe { gc_alloc_handle(&mut *self.gc, value) }
    }
    /// Perform major GC.
    pub fn major(&mut self) {
        self.gc.major();
    }
    /// Perform minor GC.
    pub fn minor(&mut self) {
        self.gc.minor();
    }
    /// Perform full GC.
    pub fn full(&mut self) {
        self.gc.full();
    }
    /// Execute write barrier.
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

/// Deferrers GC when it is alive.
#[repr(C)]
pub struct DeferGC {
    heap: *const Heap,
}

impl DeferGC {
    /// Create new `DeferGC` instance and defer GC.
    pub fn new(heap: *const Heap) -> Self {
        unsafe {
            let heap = &mut *(heap as *mut Heap);
            heap.gc.defer_gc();
        }
        Self { heap }
    }
}

impl Drop for DeferGC {
    fn drop(&mut self) {
        unsafe {
            let heap = &mut *(self.heap as *mut Heap);
            heap.gc.undefer_gc();
        }
    }
}
