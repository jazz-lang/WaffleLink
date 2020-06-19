//! Mark & Sweep garbage collector.
pub mod collector;
pub mod constants;
pub mod immix_space;
use super::object::*;
use std::alloc::Layout;
use std::cmp::Ordering;
use std::sync::atomic::{spin_loop_hint, AtomicBool, AtomicUsize, Ordering as A};
use std::sync::Arc;

pub const K: usize = 1024;
pub const M: usize = K * K;
pub const G: usize = M * K;
/// The type of collection that will be performed.
pub enum CollectionType {
    /// A simple reference counting collection.
    RCCollection,

    /// A reference counting collection with proactive opportunistic
    /// evacuation.
    RCEvacCollection,

    /// A reference counting collection followed by the immix tracing (cycle)
    /// collection.
    ImmixCollection,

    /// A reference counting collection followed by the immix tracing (cycle)
    /// collection. Both with opportunistict evacuation.
    ImmixEvacCollection,
}

impl CollectionType {
    /// Returns if this `CollectionType` is an evacuating collection.
    pub fn is_evac(&self) -> bool {
        use self::CollectionType::{ImmixEvacCollection, RCEvacCollection};
        match *self {
            RCEvacCollection | ImmixEvacCollection => true,
            _ => false,
        }
    }

    /// Returns if this `CollectionType` is a cycle collecting collection.
    pub fn is_immix(&self) -> bool {
        use self::CollectionType::{ImmixCollection, ImmixEvacCollection};
        match *self {
            ImmixCollection | ImmixEvacCollection => true,
            _ => false,
        }
    }
}
use collector::*;
use parking_lot::Mutex;
pub struct Heap {
    /// The default immix space.
    immix_space: immix_space::ImmixSpace,
    collector: Mutex<Collector>,
    /// The current live mark.
    ///
    /// During allocation of objects this value is used as the `mark` state of
    /// new objects. During allocation the value is negated and used to mark
    /// objects during the tracing mark phase. This way the newly allocated
    /// objects are always initialized with the last `mark` state with will be
    /// flipped if they are reached is the mark phase.
    current_live_mark: AtomicBool,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            current_live_mark: AtomicBool::new(false),
            immix_space: immix_space::ImmixSpace::new(),
            collector: Mutex::new(Collector::new()),
        }
    }

    pub fn is_gc_object(&self, object: GCObjectRef) -> bool {
        self.immix_space.is_gc_object(object)
    }

    pub fn allocate(&self, ty: WaffleType, size: usize) -> Option<WaffleCellPointer> {
        if size >= constants::LARGE_OBJECT {
            todo!("Large space");
        } else {
            self.immix_space.allocate(ty, size)
        }
    }

    pub fn unregister_thread() {
        use immix_space::*;
        LOCAL_ALLOCATOR.with(|alloc| {
            crate::VM
                .state
                .heap
                .immix_space
                .allocators
                .lock()
                .retain(|a| !Arc::ptr_eq(a, &*alloc))
        });
    }
    fn collect_roots(&self, threads: &[Arc<crate::thread::Thread>]) -> Vec<GCObjectRef> {
        let mut roots = vec![];
        let immix_filter = self.immix_space.is_gc_object_filter();
        for thread in threads.iter() {
            let mut scan = thread.stack_top.load(A::Relaxed);
            let mut end = thread.stack_cur.load(A::Relaxed);
            if end < scan {
                std::mem::swap(&mut end, &mut scan);
            }
            let mut scan = Address::from(scan);
            let end = Address::from(end);
            debug_assert!(scan.is_non_null());
            debug_assert!(end.is_non_null());
            while scan < end {
                // Scan for GC object.
                let frame = scan.to_ptr::<*mut u8>();
                unsafe {
                    let value = *frame;
                    if value.is_null() {
                        scan = scan.add_ptr(1);
                        continue;
                    } else {
                        let cell = WaffleCellPointer::from_ptr(value.cast());
                        //log::debug!("Try {:p} at {:p}", cell.raw(), frame);
                        if immix_filter(cell) {
                            log::trace!("Root object {:p} at {:p}", cell.raw(), frame);
                            roots.push(cell);
                        }
                    }
                }
                scan = scan.add_ptr(1);
            }
            let mut scan = thread.regs.as_ptr().cast::<u8>();
            let end = (scan as usize
                + (std::mem::size_of::<setjmp::jmp_buf>() / std::mem::size_of::<usize>())
                - 1) as *const u8;
            while scan < end {
                let frame = scan.cast::<crate::value::Value>();
                unsafe {
                    // We're dereferencing `Value` and it takes 64 bits of space so this code will not work on 32 bit machines.
                    let value = *frame;
                    if value.is_empty() || !value.is_cell() {
                        scan = scan.offset(crate::WORD as _);
                        continue;
                    } else {
                        if immix_filter(value.as_cell()) {
                            log::trace!(
                                "Root object(in reg) {:p} at {:p}",
                                value.as_cell().raw(),
                                frame
                            );
                            roots.push(value.as_cell());
                        }
                    }
                    scan = scan.offset(crate::WORD as _);
                }
            }
        }

        roots
    }
    pub fn collect(&self, threads: &[Arc<crate::thread::Thread>]) {
        let roots = self.collect_roots(threads);
        let mut collector = self.collector.lock();
        collector.extend_all_blocks(self.immix_space.get_all_blocks());
        // pin roots so GC will not move them.
        roots.iter().for_each(|item| {
            item.value_mut().header_mut().set_pinned();
        });
        let ty = collector.prepare_collection(
            true,
            false,
            self.immix_space.available_blocks(),
            self.immix_space.evac_headroom(),
        );

        collector.collect(
            &ty,
            &roots,
            &self.immix_space,
            !self.current_live_mark.load(A::Relaxed),
        );
        collector.complete_collection(&ty, &self.immix_space);
        // GC cycle finished, we can unpin roots safely.
        roots.iter().for_each(|item| {
            item.value_mut().header_mut().unpin();
        });

        if ty.is_immix() {
            self.current_live_mark.fetch_xor(true, A::Relaxed);
            self.immix_space
                .set_current_live_mark(self.current_live_mark.load(A::Relaxed));
        }
    }
}
unsafe impl Send for Heap {}
unsafe impl Sync for Heap {}

pub type GCObjectRef = WaffleCellPointer;

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
use std::mem::MaybeUninit;

pub fn retain_mut<T>(v: &mut Vec<T>, mut f: impl FnMut(&mut T) -> bool) {
    for i in (0..v.len()).rev() {
        // Process the item, determine if it should be removed
        let should_remove = {
            // Everyone take some damage! Remove the dead!
            let elem = &mut v[i];
            !f(elem)
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
#[repr(transparent)]
pub struct UnsafeCell<T: ?Sized> {
    value: T,
}

impl<T> UnsafeCell<T> {
    /// Constructs a new instance of `UnsafeCell` which will wrap the specified
    /// value.
    ///
    /// All access to the inner value through methods is `unsafe`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::UnsafeCell;
    ///
    /// let uc = UnsafeCell::new(5);
    /// ```
    #[inline]
    pub const fn new(value: T) -> UnsafeCell<T> {
        UnsafeCell { value }
    }

    /// Unwraps the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::UnsafeCell;
    ///
    /// let uc = UnsafeCell::new(5);
    ///
    /// let five = uc.into_inner();
    /// ```
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: ?Sized> UnsafeCell<T> {
    /// Gets a mutable pointer to the wrapped value.
    ///
    /// This can be cast to a pointer of any kind.
    /// Ensure that the access is unique (no active references, mutable or not)
    /// when casting to `&mut T`, and ensure that there are no mutations
    /// or mutable aliases going on when casting to `&T`
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::UnsafeCell;
    ///
    /// let uc = UnsafeCell::new(5);
    ///
    /// let five = uc.get();
    /// ```
    #[inline]

    pub const fn get(&self) -> *mut T {
        // We can just cast the pointer from `UnsafeCell<T>` to `T` because of
        // #[repr(transparent)]. This exploits libstd's special status, there is
        // no guarantee for user code that this will work in future versions of the compiler!
        self as *const UnsafeCell<T> as *const T as *mut T
    }

    /// Gets a mutable pointer to the wrapped value.
    /// The difference to [`get`] is that this function accepts a raw pointer,
    /// which is useful to avoid the creation of temporary references.
    ///
    /// The result can be cast to a pointer of any kind.
    /// Ensure that the access is unique (no active references, mutable or not)
    /// when casting to `&mut T`, and ensure that there are no mutations
    /// or mutable aliases going on when casting to `&T`.
    ///
    /// [`get`]: #method.get
    ///
    /// # Examples
    ///
    /// Gradual initialization of an `UnsafeCell` requires `raw_get`, as
    /// calling `get` would require creating a reference to uninitialized data:
    ///
    /// ```
    /// #![feature(unsafe_cell_raw_get)]
    /// use std::cell::UnsafeCell;
    /// use std::mem::MaybeUninit;
    ///
    /// let m = MaybeUninit::<UnsafeCell<i32>>::uninit();
    /// unsafe { UnsafeCell::raw_get(m.as_ptr()).write(5); }
    /// let uc = unsafe { m.assume_init() };
    ///
    /// assert_eq!(uc.into_inner(), 5);
    /// ```
    #[inline]

    pub const fn raw_get(this: *const Self) -> *mut T {
        // We can just cast the pointer from `UnsafeCell<T>` to `T` because of
        // #[repr(transparent)]. This exploits libstd's special status, there is
        // no guarantee for user code that this will work in future versions of the compiler!
        this as *const T as *mut T
    }
}

impl<T: Default> Default for UnsafeCell<T> {
    /// Creates an `UnsafeCell`, with the `Default` value for T.
    fn default() -> UnsafeCell<T> {
        UnsafeCell::new(Default::default())
    }
}

impl<T> From<T> for UnsafeCell<T> {
    fn from(t: T) -> UnsafeCell<T> {
        UnsafeCell::new(t)
    }
}

unsafe fn object_init(ty: WaffleType, addr: Address) {
    use crate::value::*;
    match ty {
        _ => (),
    }
}
