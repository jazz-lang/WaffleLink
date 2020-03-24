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

use super::arc::*;
use super::value::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Cell {
    pub forward: crate::util::mem::Address,
}
impl Cell {
    #[inline(always)]
    pub fn new() -> Self {
        unsafe { std::hint::unreachable_unchecked() }
    }

    pub fn cast<T>(&self) -> *mut T {
        unsafe { std::mem::transmute(self) }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StringCell {
    pub header: Cell,
    pub length: usize,
    pub hash: u32,
    pub value: *mut u8,
}

impl StringCell {
    pub fn length(&self) -> usize {
        unsafe { *((self as *const Self).offset(offset_of!(Self, length) as isize) as *mut usize) }
    }
    pub fn length_slot(&self) -> *mut usize {
        unsafe { (self as *const Self).offset(offset_of!(Self, length) as isize) as *mut usize }
    }

    pub fn hash_slot(&self) -> *mut u32 {
        unsafe { (self as *const Self).offset(offset_of!(Self, hash) as isize) as *mut u32 }
    }

    pub unsafe fn set_length(&self, len: usize) {
        let ptr = unsafe {
            &mut *((self as *const Self).offset(offset_of!(Self, length) as isize) as *mut usize)
        };
        *ptr = len;
    }

    pub fn value<'a>(&self) -> &'a str {
        unsafe {
            let ptr = (self as *const Self).offset(offset_of!(Self, value) as isize) as *mut u8;
            let bytes = std::slice::from_raw_parts(ptr, self.length());

            std::str::from_utf8(bytes).unwrap()
        }
    }

    pub fn hash(&self) -> u32 {
        unsafe {
            let ptr = (self as *const Self).offset(offset_of!(Self, hash) as isize) as *mut u32;
            let hash = *ptr;
            if hash == 0 {
                *ptr = fxhash::hash32(self.value())
            }

            *ptr
        }
    }
}

#[cfg(test)]
mod tests {
    extern "C" {
        fn malloc(s: usize) -> *mut u8;
    }
    use super::*;
    use std::io::Write;
    use std::mem::size_of;
    #[test]
    fn alloc_string() {
        unsafe {
            let mem = malloc(size_of::<StringCell>() + "Hello,World!\0".len()) as *mut Cell;
            (&mut *mem).forward = crate::util::mem::Address::from(42);
            let string = (&*mem).cast::<StringCell>();
            (*string).set_length("Hello,World!".len());
            std::ptr::copy_nonoverlapping(
                b"Hello,World!\0".as_ptr(),
                string.offset(crate::offset_of!(StringCell, value) as isize) as *mut u8,
                "Hello,World!\0".len(),
            );
            extern "C" {
                fn puts(raw: *mut u8);
            }
            puts(string.offset(offset_of!(StringCell, value) as isize) as *mut u8);
            assert_eq!((&*string).header.forward.to_usize(), 42);
            std::io::stderr()
                .lock()
                .write(format!("{}\n", (*string).length()).as_bytes())
                .unwrap();
            assert_eq!((*string).length(), 12);
            assert_eq!((*string).value(), "Hello,World!");
        }
    }
}
