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

use crate::util::mem::*;
use std::collections::LinkedList;
pub struct Space {
    pub top: Address,
    pub limit: Address,
    pub pages: LinkedList<Page>,
    pub size: usize,
    pub size_limit: usize,
    pub page_size: usize,
    pub allocated_size: usize,
}

impl Space {
    pub fn empty() -> Self {
        Self {
            top: Address::null(),
            limit: Address::null(),
            pages: LinkedList::new(),
            size: 0,
            allocated_size: 0,
            page_size: 0,
            size_limit: 0,
        }
    }
    pub fn new(page_size: usize) -> Self {
        let mut pages = LinkedList::new();
        let page = Page::new(page_size);
        pages.push_back(page);
        let top = Address::from_ptr(&pages.back().unwrap().top);
        let limit = Address::from_ptr(&pages.back().unwrap().limit);
        let mut space = Space {
            top,
            limit,
            pages,
            size: 0,
            page_size,
            size_limit: 0,
            allocated_size: 0,
        };
        space.compute_size_limit();
        space
    }

    pub fn compute_size_limit(&mut self) {
        self.size_limit = self.size << 1;
    }
    pub fn may_allocate_in_current(&mut self, size: usize) -> bool {
        let even_bytes = size + (size & 0x01);
        let place_in_current = self.top.deref().offset(even_bytes) <= self.limit.deref();
        place_in_current
    }
    pub fn add_page(&mut self, size: usize) {
        let real_size = align_usize(size, page_size());
        let page = Page::new(real_size);
        self.pages.push_back(page);
        let page = self.pages.back().unwrap();
        self.size += real_size;
        self.top = Address::from_ptr(&page.top);
        self.limit = Address::from_ptr(&page.limit);
    }

    pub fn fast_allocate(&mut self, bytes: usize, needs_gc: &mut bool) -> Address {
        let even_bytes = bytes + (bytes & 0x01);
        let place_in_current = self.top.deref().offset(even_bytes) < self.limit.deref();

        if !place_in_current {
            *needs_gc = true;
            log::debug!("Add new page");
            self.add_page(even_bytes);
        }
        self.allocated_size += even_bytes;
        let result = self.top.deref();
        unsafe {
            *self.top.to_mut_ptr::<*mut u8>() =
                self.top.deref().offset(even_bytes).to_mut_ptr::<u8>();
        }
        result
    }

    pub fn allocate(&mut self, bytes: usize, needs_gc: &mut bool) -> Address {
        let even_bytes = bytes + (bytes & 0x01);
        let place_in_current = self.top.deref().offset(even_bytes) < self.limit.deref();

        if !place_in_current {
            let mut iter = self.pages.iter();
            let mut head = iter.next_back();
            loop {
                if self.top.deref().offset(even_bytes) > self.limit.deref() && head.is_some() {
                    let old_head = head;
                    head = iter.next_back();
                    self.top = Address::from_ptr(&old_head.unwrap().top);
                    self.limit = Address::from_ptr(&old_head.unwrap().limit);
                } else {
                    break;
                }
            }

            if head.is_none() {
                *needs_gc = true;
                self.add_page(even_bytes);
            }
        }
        self.allocated_size += even_bytes;
        let result = self.top.deref();
        unsafe {
            *self.top.to_mut_ptr::<*mut u8>() =
                self.top.deref().offset(even_bytes).to_mut_ptr::<u8>();
        }
        result
    }

    pub fn swap(&mut self, space: &mut Space) {
        self.clear();
        while space.pages.is_empty() != true {
            self.pages.push_back(space.pages.pop_back().unwrap());
            self.size += self.pages.back().unwrap().size;
        }
        self.allocated_size = space.allocated_size;
        let page = self.pages.back().unwrap();
        self.top = Address::from_ptr(&page.top);
        self.limit = Address::from_ptr(&page.limit);
    }

    pub fn contains(&self, addr: Address) -> bool {
        for page in self.pages.iter() {
            let page: &Page = page;
            if addr >= page.data && addr <= page.limit {
                return true;
            }
        }

        false
    }

    pub fn clear(&mut self) {
        self.size = 0;
        while let Some(page) = self.pages.pop_back() {
            page.uncommit();
        }
    }
}

#[derive(Copy, Clone)]
pub struct Page {
    pub data: Address,
    pub top: Address,
    pub limit: Address,
    pub size: usize,
}

impl Page {
    pub fn new(size: usize) -> Self {
        let data = commit(size, false);
        let top = data;
        let limit = data.offset(size);
        Self {
            top,
            data,
            limit,
            size,
        }
    }

    fn uncommit(&self) {
        uncommit(self.data, self.size)
    }
}
