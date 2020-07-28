use std::cmp::*;
use std::fmt;
static mut PAGE_SIZE: usize = 0;
static mut PAGE_SIZE_BITS: usize = 0;

pub fn page_size() -> usize {
    let result = unsafe { PAGE_SIZE };

    if result != 0 {
        return result;
    }

    init_page_size();

    unsafe { PAGE_SIZE }
}

pub fn page_size_bits() -> usize {
    let result = unsafe { PAGE_SIZE_BITS };

    if result != 0 {
        return result;
    }

    init_page_size();

    unsafe { PAGE_SIZE_BITS }
}

fn init_page_size() {
    unsafe {
        PAGE_SIZE = determine_page_size();
        assert!((PAGE_SIZE & (PAGE_SIZE - 1)) == 0);

        PAGE_SIZE_BITS = log2(PAGE_SIZE);
    }
}

pub fn map_gc_mem() -> Address {
    commit(memory_limit(), false)
}

#[cfg(target_family = "unix")]
pub fn memory_limit() -> usize {
    unsafe {
        use libc::*;
        sysconf(_SC_PHYS_PAGES) as usize * sysconf(_SC_PAGESIZE) as usize
    }
}

#[cfg(target_family = "windows")]
pub(crate) fn memory_limit() -> usize {
    unimplemented!()
}

#[cfg(target_family = "unix")]
fn determine_page_size() -> usize {
    let val = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };

    if val <= 0 {
        panic!("could not determine page size.");
    }

    val as usize
}

#[cfg(target_family = "windows")]
fn determine_page_size() -> usize {
    use winapi::um::sysinfoapi::{GetSystemInfo, LPSYSTEM_INFO, SYSTEM_INFO};

    unsafe {
        let mut system_info: SYSTEM_INFO = std::mem::zeroed();
        GetSystemInfo(&mut system_info as LPSYSTEM_INFO);

        system_info.dwPageSize as usize
    }
}

/// determine log_2 of given value
fn log2(mut val: usize) -> usize {
    let mut log = 0;
    assert!(val <= u32::max_value() as usize);

    if (val & 0xFFFF0000) != 0 {
        val >>= 16;
        log += 16;
    }
    if val >= 256 {
        val >>= 8;
        log += 8;
    }
    if val >= 16 {
        val >>= 4;
        log += 4;
    }
    if val >= 4 {
        val >>= 2;
        log += 2;
    }

    log + (val >> 1)
}

#[test]
fn test_log2() {
    for i in 0..32 {
        assert_eq!(i, log2(1 << i));
    }
}
use std::i32;
use std::mem::size_of;
/// return pointer width: either 4 or 8
/// (although only 64bit architectures are supported right now)
#[inline(always)]
pub fn ptr_width() -> i32 {
    size_of::<*const u8>() as i32
}

#[inline(always)]
pub fn ptr_width_usize() -> usize {
    size_of::<*const u8>() as usize
}

/// returns true if given value is a multiple of a page size.
pub fn is_page_aligned(val: usize) -> bool {
    let align = page_size_bits();

    // we can use shifts here since we know that
    // page size is power of 2
    val == ((val >> align) << align)
}

#[test]
fn test_is_page_aligned() {
    let p = page_size();

    assert_eq!(false, is_page_aligned(1));
    assert_eq!(false, is_page_aligned(2));
    assert_eq!(false, is_page_aligned(64));
    assert_eq!(true, is_page_aligned(p));
    assert_eq!(true, is_page_aligned(2 * p));
    assert_eq!(true, is_page_aligned(3 * p));
}

/// round the given value up to the nearest multiple of a page
pub fn page_align(val: usize) -> usize {
    let align = page_size_bits();

    // we know that page size is power of 2, hence
    // we can use shifts instead of expensive division
    ((val + (1 << align) - 1) >> align) << align
}

#[test]
fn test_page_align() {
    let p = page_size();

    assert_eq!(p, page_align(1));
    assert_eq!(p, page_align(p - 1));
    assert_eq!(p, page_align(p));
    assert_eq!(2 * p, page_align(p + 1));
}

/// rounds the given value `val` up to the nearest multiple
/// of `align`
pub fn align(value: u32, align: u32) -> u32 {
    if align == 0 {
        return value;
    }

    ((value + align - 1) / align) * align
}

/// rounds the given value `val` up to the nearest multiple
/// of `align`
pub fn align_i32(value: i32, align: i32) -> i32 {
    if align == 0 {
        return value;
    }

    ((value + align - 1) / align) * align
}

/// rounds the given value `val` up to the nearest multiple
/// of `align`.
pub fn align_usize(value: usize, align: usize) -> usize {
    if align == 0 {
        return value;
    }

    ((value + align - 1) / align) * align
}

/// returns 'true' if th given `value` is already aligned
/// to `align`.
pub fn is_aligned(value: usize, align: usize) -> bool {
    align_usize(value, align) == value
}

/// returns true if value fits into u8 (unsigned 8bits).
pub fn fits_u8(value: i64) -> bool {
    0 <= value && value <= 255
}

/// returns true if value fits into i32 (signed 32bits).
pub fn fits_i32(value: i64) -> bool {
    i32::MIN as i64 <= value && value <= i32::MAX as i64
}
use super::*;
use std::ptr;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fits_u8() {
        assert_eq!(true, fits_u8(0));
        assert_eq!(true, fits_u8(255));
        assert_eq!(false, fits_u8(256));
        assert_eq!(false, fits_u8(-1));
    }

    #[test]
    fn test_fits_i32() {
        assert_eq!(true, fits_i32(0));
        assert_eq!(true, fits_i32(i32::MAX as i64));
        assert_eq!(true, fits_i32(i32::MIN as i64));
        assert_eq!(false, fits_i32(i32::MAX as i64 + 1));
        assert_eq!(false, fits_i32(i32::MIN as i64 - 1));
    }
}

#[cfg(target_family = "unix")]
pub fn reserve(size: usize) -> Address {
    debug_assert!(mem::is_page_aligned(size));

    let ptr = unsafe {
        libc::mmap(
            ptr::null_mut(),
            size,
            libc::PROT_NONE,
            libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_NORESERVE,
            -1,
            0,
        ) as *mut libc::c_void
    };

    if ptr == libc::MAP_FAILED {
        panic!("reserving memory with mmap() failed");
    }

    Address::from_ptr(ptr)
}

#[cfg(target_family = "windows")]
pub fn reserve(size: usize) -> Address {
    debug_assert!(mem::is_page_aligned(size));

    use kernel32::VirtualAlloc;
    use winapi::um::winnt::{MEM_RESERVE, PAGE_NOACCESS};

    let ptr = unsafe { VirtualAlloc(ptr::null_mut(), size as u64, MEM_RESERVE, PAGE_NOACCESS) };

    if ptr.is_null() {
        panic!("VirtualAlloc failed");
    }

    Address::from_ptr(ptr)
}

pub fn reserve_align(size: usize, align: usize) -> Address {
    debug_assert!(mem::is_page_aligned(size));
    debug_assert!(mem::is_page_aligned(align));

    let align_minus_page = align - page_size();

    let unaligned = reserve(size + align_minus_page);
    let aligned: Address = mem::align_usize(unaligned.to_usize(), align).into();

    let gap_start = aligned.offset_from(unaligned);
    let gap_end = align_minus_page - gap_start;

    if gap_start > 0 {
        uncommit(unaligned, gap_start);
    }

    if gap_end > 0 {
        uncommit(aligned.offset(size), gap_end);
    }

    aligned
}

#[cfg(target_family = "unix")]
pub fn commit(size: usize, executable: bool) -> Address {
    debug_assert!(mem::is_page_aligned(size));

    let mut prot = libc::PROT_READ | libc::PROT_WRITE;

    if executable {
        prot |= libc::PROT_EXEC;
    }

    let ptr = unsafe {
        libc::mmap(
            ptr::null_mut(),
            size,
            prot,
            libc::MAP_PRIVATE | libc::MAP_ANON,
            -1,
            0,
        )
    };

    if ptr == libc::MAP_FAILED {
        panic!("committing memory with mmap() failed");
    }

    Address::from_ptr(ptr)
}

#[cfg(target_family = "windows")]
pub fn commit(size: usize, executable: bool) -> Address {
    debug_assert!(mem::is_page_aligned(size));

    use kernel32::VirtualAlloc;
    use winapi::um::winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_READWRITE};

    let prot = if executable {
        PAGE_EXECUTE_READWRITE
    } else {
        PAGE_READWRITE
    };

    let ptr = unsafe { VirtualAlloc(ptr::null_mut(), size as u64, MEM_COMMIT | MEM_RESERVE, prot) };

    if ptr.is_null() {
        panic!("VirtualAlloc failed");
    }

    Address::from_ptr(ptr)
}

#[cfg(target_family = "unix")]
pub fn commit_at(ptr: Address, size: usize, executable: bool) {
    debug_assert!(mem::is_page_aligned(size));

    let mut prot = libc::PROT_READ | libc::PROT_WRITE;

    if executable {
        prot |= libc::PROT_EXEC;
    }

    let val = unsafe {
        libc::mmap(
            ptr.to_mut_ptr(),
            size,
            prot,
            libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_FIXED,
            -1,
            0,
        )
    };

    if val == libc::MAP_FAILED {
        panic!("committing memory with mmap() failed");
    }
}

#[cfg(target_family = "windows")]
pub fn commit_at(ptr: Address, size: usize, executable: bool) {
    debug_assert!(ptr.is_page_aligned());
    debug_assert!(mem::is_page_aligned(size));

    use kernel32::VirtualAlloc;
    use winapi::um::winnt::{MEM_COMMIT, PAGE_EXECUTE_READWRITE, PAGE_READWRITE};

    let prot = if executable {
        PAGE_EXECUTE_READWRITE
    } else {
        PAGE_READWRITE
    };

    let result = unsafe { VirtualAlloc(ptr.to_mut_ptr(), size as u64, MEM_COMMIT, prot) };

    if result != ptr.to_mut_ptr() {
        panic!("VirtualAlloc failed");
    }
}

#[cfg(target_family = "unix")]
pub fn uncommit(ptr: Address, size: usize) {
    debug_assert!(mem::is_page_aligned(size));

    let val = unsafe {
        libc::mmap(
            ptr.to_mut_ptr(),
            size,
            libc::PROT_NONE,
            libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_NORESERVE,
            -1,
            0,
        )
    };

    if val == libc::MAP_FAILED {
        panic!("uncommitting memory with mmap() failed");
    }
}

#[cfg(target_family = "windows")]
pub fn uncommit(ptr: Address, size: usize) {
    debug_assert!(ptr.is_page_aligned());
    use kernel32::VirtualFree;
    use winapi::um::winnt::MEM_RELEASE;

    let _ = unsafe { VirtualFree(ptr.to_mut_ptr(), size as _, MEM_RELEASE) };
}

#[cfg(target_family = "unix")]
pub fn discard(ptr: Address, size: usize) {
    let res = unsafe { libc::madvise(ptr.to_mut_ptr(), size, libc::MADV_DONTNEED) };

    if res != 0 {
        panic!("discarding memory with madvise() failed");
    }

    let res = unsafe { libc::mprotect(ptr.to_mut_ptr(), size, libc::PROT_NONE) };

    if res != 0 {
        panic!("discarding memory with mprotect() failed");
    }
}

#[cfg(target_family = "windows")]
pub fn discard(ptr: Address, size: usize) {
    debug_assert!(ptr.is_page_aligned());
    debug_assert!(mem::is_page_aligned(size));

    use kernel32::VirtualFree;
    use winapi::um::winnt::MEM_DECOMMIT;

    let _ = unsafe { VirtualFree(ptr.to_mut_ptr(), size as u64, MEM_DECOMMIT) };
}

#[cfg(target_family = "unix")]
pub fn protect(start: Address, size: usize, access: Access) {
    if access.is_none() {
        discard(start, size);
        return;
    }

    let protection = match access {
        Access::None => unreachable!(),
        Access::Read => libc::PROT_READ,
        Access::ReadWrite => libc::PROT_READ | libc::PROT_WRITE,
        Access::ReadExecutable => libc::PROT_READ | libc::PROT_EXEC,
        Access::ReadWriteExecutable => libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
    };

    let res = unsafe { libc::mprotect(start.to_mut_ptr(), size, protection) };

    if res != 0 {
        panic!("mprotect() failed");
    }
}

#[cfg(target_family = "windows")]
pub fn protect(start: Address, size: usize, access: Access) {
    debug_assert!(start.is_page_aligned());
    debug_assert!(mem::is_page_aligned(size));

    use kernel32::VirtualAlloc;
    use winapi::um::winnt::{
        MEM_COMMIT, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_READONLY, PAGE_READWRITE,
    };

    if access.is_none() {
        discard(start, size);
        return;
    }

    let protection = match access {
        Access::None => unreachable!(),
        Access::Read => PAGE_READONLY,
        Access::ReadWrite => PAGE_READWRITE,
        Access::ReadExecutable => PAGE_EXECUTE_READ,
        Access::ReadWriteExecutable => PAGE_EXECUTE_READWRITE,
    };

    let ptr = unsafe { VirtualAlloc(start.to_mut_ptr(), size as u64, MEM_COMMIT, protection) };

    if ptr.is_null() {
        panic!("VirtualAlloc failed");
    }
}

pub enum Access {
    None,
    Read,
    ReadWrite,
    ReadExecutable,
    ReadWriteExecutable,
}

impl Access {
    fn is_none(&self) -> bool {
        match self {
            Access::None => true,
            _ => false,
        }
    }
}

#[repr(transparent)]
pub struct Ptr<T: ?Sized>(pub(crate) *mut T);

impl<T: ?Sized> Ptr<T> {
    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.0 }
    }
}

impl<T> Ptr<T> {
    pub fn new(x: T) -> Self {
        Self(Box::into_raw(Box::new(x)))
    }

    pub fn from_box(b: Box<T>) -> Self {
        Self(Box::into_raw(b))
    }

    pub fn set(&self, val: T) {
        unsafe { self.0.write(val) };
    }

    pub fn replace(&self, val: T) -> T {
        std::mem::replace(self.get(), val)
    }

    pub fn take(&self) -> T
    where
        T: Default,
    {
        self.replace(T::default())
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

use std::hash::*;

impl<T> Hash for Ptr<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> PartialEq for Ptr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Ptr<T> {}

impl<T> Copy for Ptr<T> {}
impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> std::ops::Deref for Ptr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}

unsafe impl<T> Send for Ptr<T> {}
unsafe impl<T> Sync for Ptr<T> {}

use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicPtr, Ordering};

/// The mask to use for untagging a pointer.
const UNTAG_MASK: usize = (!0x7) as usize;

/// Returns true if the pointer has the given bit set to 1.
pub fn bit_is_set<T>(pointer: *mut T, bit: usize) -> bool {
    let shifted = 1 << bit;

    (pointer as usize & shifted) == shifted
}

/// Returns the pointer with the given bit set.
pub fn with_bit<T>(pointer: *mut T, bit: usize) -> *mut T {
    (pointer as usize | 1 << bit) as _
}

pub fn without_bit<T>(pointer: *mut T, bit: usize) -> *mut T {
    (pointer as usize ^ 1 << bit) as _
}

/// Returns the given pointer without any tags set.
pub fn untagged<T>(pointer: *mut T) -> *mut T {
    (pointer as usize & UNTAG_MASK) as _
}

/// Structure wrapping a raw, tagged pointer.
#[derive(Debug)]
#[repr(transparent)]
pub struct TaggedPointer<T: ?Sized> {
    pub raw: *mut T,
}

impl<T> TaggedPointer<T> {
    /// Returns a new TaggedPointer without setting any bits.
    pub fn new(raw: *mut T) -> TaggedPointer<T> {
        TaggedPointer { raw }
    }

    /// Returns a new TaggedPointer with the given bit set.
    pub fn with_bit(raw: *mut T, bit: usize) -> TaggedPointer<T> {
        let mut pointer = Self::new(raw);

        pointer.set_bit(bit);

        pointer
    }

    pub fn unset_bit(&mut self, bit: usize) {
        if self.bit_is_set(bit) {
            self.raw = without_bit(self.raw, bit);
        }
    }

    /// Returns a null pointer.
    pub const fn null() -> TaggedPointer<T> {
        TaggedPointer {
            raw: ptr::null::<T>() as *mut T,
        }
    }

    /// Returns the wrapped pointer without any tags.
    pub fn untagged(self) -> *mut T {
        self::untagged(self.raw)
    }

    /// Returns a new TaggedPointer using the current pointer but without any
    /// tags.
    pub fn without_tags(self) -> Self {
        Self::new(self.untagged())
    }

    /// Returns true if the given bit is set.
    pub fn bit_is_set(self, bit: usize) -> bool {
        self::bit_is_set(self.raw, bit)
    }

    /// Sets the given bit.
    pub fn set_bit(&mut self, bit: usize) {
        self.raw = with_bit(self.raw, bit);
    }

    /// Returns true if the current pointer is a null pointer.
    pub fn is_null(self) -> bool {
        self.untagged().is_null()
    }

    /// Returns an immutable to the pointer's value.
    pub fn as_ref<'a>(self) -> Option<&'a T> {
        unsafe { self.untagged().as_ref() }
    }

    /// Returns a mutable reference to the pointer's value.
    pub fn as_mut<'a>(self) -> Option<&'a mut T> {
        unsafe { self.untagged().as_mut() }
    }

    /// Atomically swaps the internal pointer with another one.
    ///
    /// This boolean returns true if the pointer was swapped, false otherwise.
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]
    pub fn compare_and_swap(&self, current: *mut T, other: *mut T) -> bool {
        self.as_atomic()
            .compare_and_swap(current, other, Ordering::AcqRel)
            == current
    }

    /// Atomically replaces the current pointer with the given one.
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]
    pub fn atomic_store(&self, other: *mut T) {
        self.as_atomic().store(other, Ordering::Release);
    }

    /// Atomically loads the pointer.
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]
    pub fn atomic_load(&self) -> *mut T {
        self.as_atomic().load(Ordering::Acquire)
    }

    /// Checks if a bit is set using an atomic load.
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]
    pub fn atomic_bit_is_set(&self, bit: usize) -> bool {
        Self::new(self.atomic_load()).bit_is_set(bit)
    }

    fn as_atomic(&self) -> &AtomicPtr<T> {
        unsafe { &*(self as *const TaggedPointer<T> as *const AtomicPtr<T>) }
    }
}

impl<T> PartialEq for TaggedPointer<T> {
    fn eq(&self, other: &TaggedPointer<T>) -> bool {
        self.raw == other.raw
    }
}

impl<T> Eq for TaggedPointer<T> {}

// These traits are implemented manually as "derive" doesn't handle the generic
// "T" argument very well.
impl<T> Clone for TaggedPointer<T> {
    fn clone(&self) -> TaggedPointer<T> {
        TaggedPointer::new(self.raw)
    }
}

impl<T> Copy for TaggedPointer<T> {}

impl<T> Hash for TaggedPointer<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}
