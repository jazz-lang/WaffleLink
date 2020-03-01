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

use super::freelist::*;
use super::space::Space;
use crate::util::mem::Address;
pub struct FreeListAllocator {
    pub space: Space,
    pub freelist: FreeList,
}

impl FreeListAllocator {
    pub fn new(heap: Space) -> Self {
        Self {
            space: heap,
            freelist: FreeList::new(),
        }
    }

    pub fn allocate(&mut self, size: usize, needs_gc: &mut bool) -> Address {
        log::trace!("allocation with size {} requested", size);
        if self.space.may_allocate_in_current(size) {
            log::trace!("memory for allocation found in current page");
            // if it possible to allocate in current page we should do it
            return self.space.fast_allocate(size, &mut false);
        }
        log::trace!("no memory in current page, trying to use freelist");
        // We cannot allocate in current page, let's try to find free slot.
        let (free_space, size) = self.freelist.alloc(size);

        if free_space.is_non_null() {
            let object = free_space.addr();
            let free_size = size;
            assert!(size <= free_size);

            let free_start = object.offset(size);
            let free_end = object.offset(free_size);
            let new_free_size = free_end.offset_from(free_start);
            if new_free_size != 0 {
                self.freelist.add(free_start, new_free_size);
            }
            log::trace!("free list slot found");
            return object;
        }
        log::trace!("no free slot found");
        *needs_gc = true;
        Address::null()
    }

    pub fn free(&mut self, pointer: Address, size: usize) {
        self.freelist.add(pointer.sub(size), size);
    }
}
