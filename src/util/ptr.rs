/*
*   Copyright (c) 2020 Adel Prokurov
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/

use std::ops::{Deref, DerefMut};
pub struct Ptr<T: ?Sized> {
    pub raw: *mut T,
}

impl<T: Sized> Ptr<T> {
    pub fn new(x: T) -> Self {
        Self {
            raw: std::boxed::Box::into_raw(std::boxed::Box::new(x)),
        }
    }
    pub fn take(&self) -> T
    where
        T: Default,
    {
        assert!(!self.raw.is_null());
        unsafe { std::ptr::replace(self.raw, Default::default()) }
    }

    pub fn replace(&self, with: T) -> T {
        unsafe { std::ptr::replace(self.raw, with) }
    }

    pub fn read(&self) -> T {
        unsafe { std::ptr::read(self.raw) }
    }

    pub fn write(&self, value: T) {
        unsafe { std::ptr::write(self.raw, value) }
    }
    pub const fn null() -> Self {
        Self {
            raw: std::ptr::null_mut(),
        }
    }
}

impl<T: ?Sized> Ptr<T> {
    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.raw }
    }

    pub fn is_null(self) -> bool {
        self.raw.is_null()
    }
}

impl<T> Deref for Ptr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}

impl<T> DerefMut for Ptr<T> {
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
