use crate::*;
use std::cmp::{Ord, Ordering as FOrdering, PartialOrd};
use std::fmt;
use std::sync::atomic::Ordering;
#[cfg(target_family = "windows")]
use winapi::um::sysinfoapi::*;

pub mod bitmap;
pub mod block;
pub mod block_directory;
pub mod block_set;
pub mod constants;
pub mod precise_allocation;
pub mod tiny_bloom_filter;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Address(usize);

impl Address {
    #[inline(always)]
    pub fn to_mut_obj(self) -> &'static mut RawGc {
        unsafe { &mut *self.to_mut_ptr::<RawGc>() }
    }
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
        Address(self.0 + words * core::mem::size_of::<usize>())
    }

    #[inline(always)]
    pub fn sub_ptr(self, words: usize) -> Address {
        Address(self.0 - words * core::mem::size_of::<usize>())
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
    fn partial_cmp(&self, other: &Address) -> Option<FOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for Address {
    fn cmp(&self, other: &Address) -> FOrdering {
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

pub struct FormattedSize {
    size: usize,
}

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

pub fn formatted_size(size: usize) -> FormattedSize {
    FormattedSize { size }
}
pub const fn round_down(x: u64, n: u64) -> u64 {
    x & !n
}

pub const fn round_up(x: u64, n: u64) -> u64 {
    round_down(x + n - 1, n)
}
impl RawGc {
    pub fn as_dyn(&self) -> &'static mut dyn HeapObject {
        #[repr(C)]
        struct Obj {
            vtable: usize,
            data: usize,
        }
        unsafe {
            std::mem::transmute(Obj {
                vtable: self.vtable(),
                data: self.data() as _,
            })
        }
    }

    pub fn object_size(&self) -> usize {
        self.as_dyn().heap_size() + core::mem::size_of::<Self>()
    }

    pub fn data(&self) -> *mut u8 {
        unsafe {
            (self as *const Self as *const u8).offset(core::mem::size_of::<Self>() as _) as *mut u8
        }
    }
}

pub trait HeapObject: mopa::Any {
    fn visit_references(&self) {
        // no-op by default
    }

    fn heap_size(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

mopa::mopafy!(HeapObject);

#[repr(C)]
struct TraitObject {
    data: *mut (),
    vtable: *mut (),
}

pub fn object_ty_of<T: HeapObject>(x: *const T) -> usize {
    unsafe { std::mem::transmute::<_, TraitObject>(x as *const dyn HeapObject).vtable as _ }
}

pub fn object_ty_of_type<T: HeapObject + Sized>() -> usize {
    let result = object_ty_of(std::ptr::null::<T>());

    result
}

use std::marker::PhantomData;
use std::ptr::NonNull;
pub const GC_WHITE: u8 = 0x0;
pub const GC_BLACK: u8 = 0x1;
pub const GC_GRAY: u8 = 0x2;

#[repr(C)]
pub struct RawGc {
    vtable: u64,
}

pub struct Gc<T: HeapObject> {
    ptr: NonNull<RawGc>,
    marker: PhantomData<T>,
}
use std::ops::{Deref, DerefMut};

impl<T: HeapObject> Deref for Gc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*(&mut *self.ptr.as_ptr()).data().cast::<T>() }
    }
}

impl<T: HeapObject> DerefMut for Gc<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(&mut *self.ptr.as_ptr()).data().cast::<T>() }
    }
}

impl RawGc {
    /// Return true if this object is precie allocation
    pub fn is_precise_allocation(&self) -> bool {
        crate::heap::precise_allocation::PreciseAllocation::is_precise(self as *const _ as *mut _)
    }
    /// Return precise allocation from this object
    pub fn precise_allocation(&self) -> *mut crate::heap::precise_allocation::PreciseAllocation {
        crate::heap::precise_allocation::PreciseAllocation::from_cell(self as *const _ as *mut _)
    }
    /// Return block where this cell was allocated
    pub fn block(&self) -> *mut crate::heap::block::Block {
        crate::heap::block::Block::from_cell(crate::heap::Address::from_ptr(self))
    }
    pub fn vtable(&self) -> usize {
        (self.vtable & (!0x03)) as usize
    }
    pub fn tag(&self) -> u8 {
        (self.vtable & 0x03) as _
    }

    pub fn load_vtable(&self) -> usize {
        (self.vtable.into_atomic().load(Ordering::Relaxed) & (!0x03)) as _
    }

    pub fn load_tag(&self) -> u8 {
        (self.vtable.into_atomic().load(Ordering::Relaxed) & 0x03) as _
    }

    pub fn store_tag(&self, tag: u8) -> u8 {
        let mut old_word;
        let entry = self.vtable.into_atomic();
        while {
            old_word = entry.load(Ordering::Relaxed);
            if old_word & 0x03 == tag as u64 {
                return tag;
            }
            entry.compare_exchange(
                old_word,
                old_word | tag as u64,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) != Ok(old_word)
        } {}
        return (old_word & 0x03) as _;
    }

    pub unsafe fn set_vtable(&mut self, vtable: usize) {
        self.vtable = vtable as u64 | self.tag() as u64;
    }

    pub unsafe fn set_tag(&mut self, tag: u8) {
        self.vtable = self.vtable() as u64 | tag as u64;
    }
}

#[cfg(any(target_pointer_width = "32", feature = "tag-field"))]
impl RawGc {}

pub static PAGESIZE: once_cell::sync::Lazy<usize> = once_cell::sync::Lazy::new(|| unsafe {
    #[cfg(target_family = "windows")]
    {
        let mut si: SYSTEM_INFO = std::mem::MaybeUninit::zeroed().assume_init();
        GetSystemInfo(&mut si);
        si.dwPageSize as _
    }
    #[cfg(target_family = "unix")]
    {
        let page_size = libc::sysconf(libc::_SC_PAGESIZE);
        page_size as _
    }
});
