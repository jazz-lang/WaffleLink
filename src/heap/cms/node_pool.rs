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
