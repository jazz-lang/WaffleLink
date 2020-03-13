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

//! This module implements rooting code that allows using cells in native code.
use std::sync::atomic::{AtomicBool, Ordering};

use crate::runtime::cell::CellPointer;

/// All cell objects stored on *native* code stack (i.e local variables and function parameters in native Rust code)
/// must use the `RootedCell` type. This is RAII structure returned from Process::allocate or when you load some variable
/// from register or Waffle stack.
/// Arc and Rc is also possible to use with RootedCell.
pub struct RootedCell {
    pub(crate) inner: *mut RootedInner,
}

pub struct RootedInner {
    pub(crate) rooted: AtomicBool,
    pub(crate) inner: CellPointer,
}

impl Drop for RootedCell {
    fn drop(&mut self) {
        unsafe {
            debug_assert!(!self.inner.is_null());
            let inner = &mut *self.inner;
            inner.rooted.store(false, Ordering::Relaxed);
        }
    }
}

impl RootedCell {
    pub fn inner(&self) -> &mut RootedInner {
        unsafe { &mut *self.inner }
    }
    pub fn trace<F>(&self, mut cb: F)
    where
        F: FnMut(*const CellPointer),
    {
        cb(&self.inner().inner);
    }

    pub fn as_cell(&self) -> CellPointer {
        self.inner().inner
    }

    pub fn is_rooted(&self) -> bool {
        self.inner().rooted.load(Ordering::Relaxed)
    }
}

use std::fmt::Display;
use std::ops::{Deref, DerefMut};

impl Deref for RootedCell {
    type Target = CellPointer;
    fn deref(&self) -> &Self::Target {
        &self.inner().inner
    }
}

impl DerefMut for RootedCell {
    fn deref_mut(&mut self) -> &mut CellPointer {
        &mut self.inner().inner
    }
}
