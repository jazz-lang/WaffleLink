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

use crate::util::mem::Address;
use crate::util::ptr::Ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
pub struct Stub {
    pub start: Address,
    pub size: usize,
    pub next: Ptr<Stub>,
    pub prev: Ptr<Stub>,
}

impl Stub {
    pub fn new(start: Address, size: usize) -> Self {
        Self {
            start,
            size,
            next: Ptr::null(),
            prev: Ptr::null(),
        }
    }
}

pub struct StubList {
    head: Ptr<Stub>,
    tail: Ptr<Stub>,
}

impl StubList {
    pub fn new() -> Self {
        Self {
            head: Ptr::null(),
            tail: Ptr::null(),
        }
    }
    pub fn append(&mut self, sl: &mut Self) {
        if !self.tail.is_null() {
            self.tail.get().next = sl.head;
            if !sl.head.is_null() {
                sl.head.get().prev = self.tail;
            }
        } else {
            self.head = sl.head;
        }

        self.tail = sl.tail;
        sl.head = Ptr::null();
        sl.tail = Ptr::null();
    }

    pub fn reset(&mut self) {
        self.head = Ptr::null();
        self.tail = Ptr::null();
    }

    pub fn atomic_vacate_append(&mut self, sl: AtomicPtr<StubList>) {
        if self.head.is_null() {
            return;
        }
        let mut copy_sl = Ptr::new(Self {
            head: self.head,
            tail: self.tail,
        });
        self.head = Ptr::null();
        self.tail = Ptr::null();

        let mut atomic_sl = Ptr {
            raw: sl.swap(std::ptr::null_mut(), Ordering::Relaxed),
        };
        loop {
            if !atomic_sl.is_null() {
                copy_sl.append(&mut atomic_sl);
            }

            copy_sl = Ptr {
                raw: sl.swap(copy_sl.raw, Ordering::Relaxed),
            };

            if copy_sl.is_null() == false {
                atomic_sl = Ptr {
                    raw: sl.swap(std::ptr::null_mut(), Ordering::Relaxed),
                };
            } else {
                break;
            }
        }
    }

    pub fn erase(&mut self, mut stub: Ptr<Stub>) -> Ptr<Stub> {
        if !stub.prev.is_null() {
            stub.prev.next = stub.next;
        }

        if !stub.next.is_null() {
            stub.next.prev = stub.prev;
        }

        stub.prev = Ptr::null();
        stub.next = stub.prev;
        return stub;
    }

    pub fn push_front(&mut self, mut st: Ptr<Stub>) {
        st.next = self.head;
        if !self.head.is_null() {
            self.head.prev = st;
        } else {
            self.tail = st;
        }

        self.head = st;
        st.prev = Ptr::null();
    }

    pub fn push_back(&mut self, mut st: Ptr<Stub>) {
        st.prev = self.tail;
        if !self.tail.is_null() {
            self.tail.next = st;
        } else {
            self.head = st;
        }
        self.tail = st;
        st.next = Ptr::null();
    }

    pub fn front(&self) -> Ptr<Stub> {
        self.head
    }

    pub fn back(&self) -> Ptr<Stub> {
        self.tail
    }

    pub fn pop_front(&mut self) {
        let mut st = self.head;
        if !self.head.next.is_null() {
            self.head.next.prev = Ptr::null()
        } else {
            self.tail = Ptr::null();
        }
        self.head = self.head.next;
        st.next = Ptr::null();
    }

    pub fn pop_back(&mut self) {
        let mut st = self.tail;
        if !self.tail.prev.is_null() {
            self.tail.prev.next = Ptr::null();
        } else {
            self.head = Ptr::null();
        }

        self.tail = st.prev;
        st.prev = Ptr::null();
    }

    pub fn empty(&self) -> bool {
        self.head.is_null()
    }
}

use super::node_pool::List;

impl List for StubList {
    type NodeType = Stub;
    fn pop_back_(&mut self) -> Ptr<Stub> {
        if self.back().is_null() {
            return Ptr::null();
        }
        let r = self.back();
        self.pop_back();
        r
    }

    fn pop_front_(&mut self) -> Ptr<Stub> {
        if self.front().is_null() {
            return Ptr::null();
        }
        let r = self.front();
        self.pop_front();
        r
    }

    fn push_back_(&mut self, st: Ptr<Self::NodeType>) {
        self.push_back(st);
    }
    fn push_front_(&mut self, st: Ptr<Self::NodeType>) {
        self.push_front(st);
    }

    fn is_empty(&self) -> bool {
        self.empty()
    }

    fn node_size(&self) -> usize {
        std::mem::size_of::<Stub>()
    }
}
