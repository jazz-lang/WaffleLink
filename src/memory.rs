use std::cmp::*;
use std::fmt;
use std::sync::atomic::{AtomicPtr, Ordering};
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
    debug_assert!(memory::is_page_aligned(size));

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
    debug_assert!(memory::is_page_aligned(size));

    use kernel32::VirtualAlloc;
    use winapi::um::winnt::{MEM_RESERVE, PAGE_NOACCESS};

    let ptr = unsafe { VirtualAlloc(ptr::null_mut(), size as u64, MEM_RESERVE, PAGE_NOACCESS) };

    if ptr.is_null() {
        panic!("VirtualAlloc failed");
    }

    Address::from_ptr(ptr)
}

pub fn reserve_align(size: usize, align: usize) -> Address {
    debug_assert!(memory::is_page_aligned(size));
    debug_assert!(memory::is_page_aligned(align));

    let align_minus_page = align - page_size();

    let unaligned = reserve(size + align_minus_page);
    let aligned: Address = memory::align_usize(unaligned.to_usize(), align).into();

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
    debug_assert!(memory::is_page_aligned(size));

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
    debug_assert!(memory::is_page_aligned(size));

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
    debug_assert!(ptr.is_page_aligned());
    debug_assert!(memory::is_page_aligned(size));

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
    debug_assert!(memory::is_page_aligned(size));

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
    debug_assert!(ptr.is_page_aligned());
    debug_assert!(memory::is_page_aligned(size));

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
    debug_assert!(memory::is_page_aligned(size));

    use kernel32::VirtualFree;
    use winapi::um::winnt::MEM_RELEASE;

    let _ = unsafe { VirtualFree(ptr.to_mut_ptr(), size as _, MEM_RELEASE) };
}

#[cfg(target_family = "unix")]
pub fn discard(ptr: Address, size: usize) {
    debug_assert!(ptr.is_page_aligned());
    debug_assert!(memory::is_page_aligned(size));

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
    debug_assert!(memory::is_page_aligned(size));

    use kernel32::VirtualFree;
    use winapi::um::winnt::MEM_DECOMMIT;

    let _ = unsafe { VirtualFree(ptr.to_mut_ptr(), size as u64, MEM_DECOMMIT) };
}

#[cfg(target_family = "unix")]
pub fn protect(start: Address, size: usize, access: Access) {
    debug_assert!(start.is_page_aligned());
    debug_assert!(memory::is_page_aligned(size));

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
    debug_assert!(memory::is_page_aligned(size));

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

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Address(pub usize);

impl Address {
    #[inline(always)]
    pub const fn from(val: usize) -> Address {
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
        Address(self.0 + words * memory::ptr_width_usize())
    }

    #[inline(always)]
    pub fn sub_ptr(self, words: usize) -> Address {
        Address(self.0 - words * memory::ptr_width_usize())
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

    #[inline(always)]
    pub fn align_page(self) -> Address {
        memory::page_align(self.to_usize()).into()
    }

    #[inline(always)]
    pub fn align_page_down(self) -> Address {
        Address(self.0 & !(memory::page_size() - 1))
    }

    #[inline(always)]
    pub fn is_page_aligned(self) -> bool {
        memory::is_page_aligned(self.to_usize())
    }

    #[inline(always)]
    pub const fn and(self, x: Address) -> Self {
        Self(self.0 & x.0)
    }

    #[inline(always)]
    pub const fn or(self, x: Address) -> Self {
        Self(self.0 | x.0)
    }

    pub fn deref(&self) -> Address {
        unsafe {
            assert!(self.is_non_null());
            Address::from_ptr(*self.to_ptr::<*const u8>())
        }
    }
    /// Atomically replaces the current pointer with the given one.
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]
    pub fn atomic_store(&self, other: *mut u8) {
        self.as_atomic().store(other, Ordering::Release);
    }

    /// Atomically loads the pointer.
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]
    pub fn atomic_load(&self) -> *mut u8 {
        self.as_atomic().load(Ordering::Acquire)
    }

    fn as_atomic(&self) -> &AtomicPtr<u8> {
        unsafe { &*(self as *const Address as *const AtomicPtr<u8>) }
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
    fn partial_cmp(&self, other: &Address) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Address {
    fn cmp(&self, other: &Address) -> std::cmp::Ordering {
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

#[cfg(target_family = "unix")]
pub fn fast_aligned_malloc(alignment: usize, size: usize) -> *mut u8 {
    let mut p = std::ptr::null_mut();
    unsafe {
        libc::posix_memalign(&mut p, alignment, size);
        if p.is_null() {
            panic!("Aligned malloc failed");
        }

        p as *mut u8
    }
}

#[cfg(target_family = "windows")]
pub fn fast_aligned_malloc(alignment: usize, size: usize) -> *mut u8 {
    unsafe {
        extern "C" {
            fn _algined_malloc(_: usize, _: usize) -> *mut u8;
        }

        let p = _aligned_malloc(size, alignment);
        if p.is_null() {
            panic!("Alligned malloc failed");
        }

        p
    }
}
