pub mod object;
pub mod segregated_freelist;

use std::cmp::Ordering;
use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Address(usize);

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
    pub const fn offset(self, offset: usize) -> Address {
        Address(self.0 + offset)
    }

    #[inline(always)]
    pub const fn sub(self, offset: usize) -> Address {
        Address(self.0 - offset)
    }

    #[inline(always)]
    pub const fn add_ptr(self, words: usize) -> Address {
        Address(self.0 + words * core::mem::size_of::<usize>())
    }

    #[inline(always)]
    pub const fn sub_ptr(self, words: usize) -> Address {
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
    pub const fn null() -> Address {
        Address(0)
    }

    #[inline(always)]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn is_non_null(self) -> bool {
        self.0 != 0
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:p}", self.to_ptr::<()>())
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

impl fmt::Pointer for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.0 as *mut ())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.0 as *mut ())
    }
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

pub const fn round_up_to_multiple_of(divisor: usize, x: usize) -> usize {
    (x + (divisor - 1)) & !(divisor - 1)
}
/// Allocates `size` Bytes aligned to `align` Bytes. Returns a null pointer on allocation failure.
///
/// The returned pointer must be deallocated by using `aligned_free`.
///
/// Note: This function is meant to be used for infrequent large allocations (as `malloc` already
/// guarantees suitable alignment for all native datatypes) and might be quite slow when used
/// heavily.
///
/// # Parameters
///
/// * `size`: The size of the allocation in bytes.
/// * `align`: The alignment of the allocation (at least the size of `usize` on the current
///   platform). Must also be a power of two.
#[inline]
pub fn aligned_alloc(size: usize, align: usize) -> *mut () {
    imp::aligned_alloc(size, align)
}

/// Deallocates aligned memory that was allocated with `aligned_alloc`. Unsafe because calling this
/// with a pointer that was not allocated with `aligned_alloc` (or already released) causes
/// undefined behavior.
#[inline]
pub unsafe fn aligned_free(ptr: *mut ()) {
    imp::aligned_free(ptr)
}

#[cfg(unix)]
mod imp {

    use libc::{c_int, c_void, free, size_t, EINVAL, ENOMEM};

    use std::{mem, ptr};

    extern "C" {
        fn posix_memalign(memptr: *mut *mut c_void, alignment: size_t, size: size_t) -> c_int;
    }

    pub fn aligned_alloc(size: usize, align: usize) -> *mut () {
        let mut memptr: *mut c_void = ptr::null_mut();
        let result = unsafe { posix_memalign(&mut memptr, align as size_t, size as size_t) };
        match result {
            0 => return memptr as *mut (),
            EINVAL => {
                if align < mem::size_of::<usize>() {
                    panic!(
                        "EINVAL: invalid alignment: {} (minimum is {})",
                        align,
                        mem::size_of::<usize>()
                    );
                }
                if !align.is_power_of_two() {
                    panic!(
                        "EINVAL: invalid alignment: {} (must be a power of two)",
                        align
                    )
                }
                panic!("EINVAL: invalid alignment: {}", align);
            }
            ENOMEM => return ptr::null_mut(),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub unsafe fn aligned_free(ptr: *mut ()) {
        free(ptr as *mut c_void)
    }
}

#[cfg(windows)]
mod imp {

    use kernel32::{GetLastError, GetSystemInfo, VirtualAlloc, VirtualFree};
    use winapi::{
        LPVOID, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_NOACCESS, PAGE_READWRITE, SIZE_T,
        SYSTEM_INFO,
    };

    use std::mem;
    use std::ptr;
    use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

    static PAGE_SIZE: AtomicUsize = ATOMIC_USIZE_INIT;

    #[cold]
    fn get_page_size() {
        let mut info: SYSTEM_INFO = unsafe { mem::uninitialized() };
        unsafe {
            GetSystemInfo(&mut info);
        }

        PAGE_SIZE.store(info.dwPageSize as usize, Ordering::SeqCst);
    }

    pub fn aligned_alloc(size: usize, align: usize) -> *mut () {
        assert!(align.is_power_of_two(), "align must be a power of two");
        assert!(
            align >= mem::size_of::<usize>(),
            "align must be at least as large as a usize"
        );

        if PAGE_SIZE.load(Ordering::SeqCst) == 0 {
            get_page_size();
        }

        unsafe {
            if align <= PAGE_SIZE.load(Ordering::SeqCst) {
                // Page alignment is guaranteed by `VirtualAlloc`
                let ptr = VirtualAlloc(
                    ptr::null_mut(),
                    size as SIZE_T,
                    MEM_COMMIT | MEM_RESERVE,
                    PAGE_READWRITE,
                );
                return ptr as *mut ();
            }

            // Step 1: Reserve `size+align-1` Bytes of address space to find a suitable address
            let ptr = VirtualAlloc(
                ptr::null_mut(),
                (size + align - 1) as SIZE_T,
                MEM_RESERVE,
                PAGE_NOACCESS,
            );
            if ptr.is_null() {
                return ptr::null_mut();
            }

            // Step 2: Calculate an aligned address within the reserved range
            // (this works because `align` must be a power of two)
            let aligned_ptr = (ptr as usize + align - 1) & !(align - 1);

            // Step 3: Actually allocate (commit) the memory
            let res = VirtualFree(ptr as LPVOID, 0, MEM_RELEASE);
            if res == 0 {
                panic!(
                    "WINAPI error {} while freeing reserved memory",
                    GetLastError()
                );
            }
            let ptr = VirtualAlloc(
                aligned_ptr as LPVOID,
                size as SIZE_T,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            );
            ptr as *mut ()
        }
    }

    pub unsafe fn aligned_free(ptr: *mut ()) {
        let res = VirtualFree(ptr as LPVOID, 0, MEM_RELEASE);
        if res == 0 {
            panic!("WINAPI error {} while releasing memory", GetLastError());
        }
    }
}
