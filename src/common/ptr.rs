use std::ops::{Deref, DerefMut};
pub struct Ptr<T: ?Sized> {
    pub raw: *mut T,
}

impl<T: Sized> Ptr<T> {
    #[inline(always)]
    pub fn from_raw<U: Sized>(x: *mut U) -> Self {
        Self { raw: x as *mut T }
    }
    #[inline(always)]
    pub fn new(x: T) -> Self {
        Self {
            raw: std::boxed::Box::into_raw(std::boxed::Box::new(x)),
        }
    }
    #[inline(always)]
    pub fn offset_bytes(self, x: isize) -> Self {
        unsafe {
            Self {
                raw: self.raw.cast::<u8>().offset(x as _).cast(),
            }
        }
    }
    #[inline(always)]

    pub fn cast<U>(self) -> Ptr<U> {
        Ptr {
            raw: self.raw as *mut U,
        }
    }
    #[inline(always)]
    pub fn offset(&self, x: isize) -> Self {
        Self {
            raw: (self.raw as isize + x) as *mut T,
        }
    }
    #[inline(always)]
    pub fn take(&self) -> T
    where
        T: Default,
    {
        assert!(!self.raw.is_null());
        unsafe { std::ptr::replace(self.raw, Default::default()) }
    }
    #[inline(always)]
    pub fn replace(&self, with: T) -> T {
        unsafe { std::ptr::replace(self.raw, with) }
    }
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { std::ptr::read(self.raw) }
    }
    #[inline(always)]
    pub fn write(&self, value: T) {
        unsafe { std::ptr::write(self.raw, value) }
    }
    #[inline(always)]
    pub const fn null() -> Self {
        Self {
            raw: std::ptr::null_mut(),
        }
    }
}
use std::ops::{Index, IndexMut};
impl<T> Index<usize> for Ptr<T> {
    type Output = T;
    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        //self.offset(index as _).get()
        unsafe { &*self.raw.offset(index as isize) }
    }
}
impl<T> IndexMut<usize> for Ptr<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut T {
        unsafe { &mut *self.raw.offset(index as isize) }
    }
}

impl<T: ?Sized> Ptr<T> {
    #[inline(always)]
    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.raw }
    }
    #[inline(always)]
    pub fn is_null(self) -> bool {
        self.raw.is_null()
    }
}

impl<T> Deref for Ptr<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        self.get()
    }
}

impl<T> DerefMut for Ptr<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        self.get()
    }
}

impl<T> Copy for Ptr<T> {}
impl<T> Clone for Ptr<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

unsafe impl<T> Send for Ptr<T> {}
unsafe impl<T> Sync for Ptr<T> {}

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

use std::hash::{Hash, Hasher};

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
pub struct TaggedPointer<T> {
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
