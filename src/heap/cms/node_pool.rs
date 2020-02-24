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
pub trait List {
    type NodeType: Sized;
    fn pop_back_(&mut self) -> Ptr<Self::NodeType>;
    fn pop_front_(&mut self) -> Ptr<Self::NodeType>;
    fn push_front_(&mut self, st: Ptr<Self::NodeType>);
    fn push_back_(&mut self, st: Ptr<Self::NodeType>);
    fn is_empty(&self) -> bool;
    fn node_size(&self) -> usize;
}

pub struct NodePool<T: List> {
    pool: T,
    offset: usize,
    allocation_dump: std::collections::LinkedList<Address>,
    chunk: Address,
}

impl<T: List> NodePool<T> {
    pub fn new(pool: T) -> Self {
        Self {
            pool,
            offset: 0,
            chunk: Address::null(),
            allocation_dump: std::collections::LinkedList::new(),
        }
    }

    pub fn get(&mut self) -> Address {
        let sz: usize = super::POOL_CHUNK_SIZE * self.pool.node_size()
            + std::mem::size_of::<super::atomic_list::ListNode<Address>>();

        if self.pool.is_empty() {
            if self.chunk.is_null() || self.offset == sz {
                self.chunk = Address::from_ptr(unsafe {
                    std::alloc::alloc(
                        std::alloc::Layout::from_size_align(
                            std::mem::size_of::<super::atomic_list::ListNode<Address>>(),
                            sz,
                        )
                        .unwrap(),
                    )
                });
            }
            self.offset = std::mem::size_of::<super::atomic_list::ListNode<Address>>();

            let node = self.chunk.offset(self.offset).to_usize();
            self.offset += self.pool.node_size();
            return Address::from(node);
        } else {
            unsafe { std::mem::transmute(self.pool.pop_front_()) }
        }
    }

    pub fn put(&mut self, node: Address) {
        self.pool.push_front_(unsafe { std::mem::transmute(node) });
    }
}
