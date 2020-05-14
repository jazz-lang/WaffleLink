use std::ops::{Deref, DerefMut};
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
pub struct DerefPointer<T> {
    /// The underlying raw pointer.
    pub pointer: *mut T,
}

unsafe impl<T> Sync for DerefPointer<T> {}
unsafe impl<T> Send for DerefPointer<T> {}

impl<T> DerefPointer<T> {
    pub fn new(value: &T) -> Self {
        DerefPointer {
            pointer: value as *const T as *mut T,
        }
    }

    pub fn from_pointer(value: *mut T) -> Self {
        DerefPointer { pointer: value }
    }
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.pointer }
    }
    pub fn get(&self) -> &T {
        unsafe { &mut *self.pointer }
    }
    pub fn null() -> Self {
        DerefPointer {
            pointer: ptr::null_mut(),
        }
    }

    pub fn is_null(self) -> bool {
        self.pointer.is_null()
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
    pub fn atomic_load(&self) -> Self {
        DerefPointer {
            pointer: self.as_atomic().load(Ordering::Acquire),
        }
    }

    fn as_atomic(&self) -> &AtomicPtr<T> {
        unsafe { &*(self as *const DerefPointer<T> as *const AtomicPtr<T>) }
    }
}

impl<T> Deref for DerefPointer<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.pointer }
    }
}

impl<T> DerefMut for DerefPointer<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.pointer as *mut T) }
    }
}

impl<T> Clone for DerefPointer<T> {
    fn clone(&self) -> DerefPointer<T> {
        DerefPointer {
            pointer: self.pointer,
        }
    }
}

impl<T> Copy for DerefPointer<T> {}

impl<T> PartialEq for DerefPointer<T> {
    fn eq(&self, other: &DerefPointer<T>) -> bool {
        self.pointer == other.pointer
    }
}

impl<T> Eq for DerefPointer<T> {}
