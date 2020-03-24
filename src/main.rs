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

#[macro_use]
extern crate waffle;

use waffle::runtime::cell::*;
extern "C" {
    fn malloc(size: usize) -> *mut u8;
}

use std::mem::size_of;

fn main() {
    unsafe {
        let mem = malloc(size_of::<StringCell>() + "Hello,World!\0".len()) as *mut Cell;
        (&mut *mem).forward = waffle::util::mem::Address::from(42);
        let string = (&*mem).cast::<StringCell>();
        std::ptr::copy_nonoverlapping(
            b"Hello,World\0".as_ptr(),
            string.offset(waffle::offset_of!(StringCell, value) as isize) as *mut u8,
            "Hello,World!\0".len(),
        );
        extern "C" {
            fn puts(raw: *mut u8);
        }
        puts(string.offset(offset_of!(StringCell, value) as isize) as *mut u8);
        println!("{}", (&*string).header.forward.to_usize());
    }
}
