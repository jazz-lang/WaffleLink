pub fn rstrerror(i: errno::Errno) -> &'static str {
    unsafe {
        std::ffi::CStr::from_ptr(libc::strerror(i.0))
            .to_str()
            .unwrap()
    }
}

pub unsafe fn zeroed<T>() -> T {
    std::mem::MaybeUninit::<T>::zeroed().assume_init()
}

use core::cell::UnsafeCell;
use core::ptr;
pub use std::ptr::{null, null_mut};

/// Just like [`Cell`] but with [volatile] read / write operations
///
/// [`Cell`]: https://doc.rust-lang.org/std/cell/struct.Cell.html
/// [volatile]: https://doc.rust-lang.org/std/ptr/fn.read_volatile.html
pub struct VolatileCell<T> {
    value: UnsafeCell<T>,
}

impl<T> VolatileCell<T> {
    /// Creates a new `VolatileCell` containing the given value
    pub const fn new(value: T) -> Self {
        VolatileCell {
            value: UnsafeCell::new(value),
        }
    }

    /// Returns a copy of the contained value
    #[inline(always)]
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        unsafe { ptr::read_volatile(self.value.get()) }
    }

    /// Sets the contained value
    #[inline(always)]
    pub fn set(&self, value: T)
    where
        T: Copy,
    {
        unsafe { ptr::write_volatile(self.value.get(), value) }
    }

    /// Returns a raw pointer to the underlying data in the cell
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut T {
        self.value.get()
    }
}

unsafe impl<T> Sync for VolatileCell<T> {}
unsafe impl<T> Send for VolatileCell<T> {}
