use super::block_info::BlockInfo;
use crate::gc::lflist::LockFreeList;
use crate::gc::{constants::*, GCObjectRef};
use dashmap::DashSet;
use parking_lot::Mutex;
use std::alloc::{alloc_zeroed, dealloc, Layout};
pub const BLOCK_LAYOUT: Layout =
    unsafe { Layout::from_size_align_unchecked(BLOCK_SIZE, BLOCK_SIZE) };
pub mod blocking_allocator {
    use super::*;
    /// The `BlockAllocator` is the global resource for blocks for the immix
    /// space.
    ///
    /// On initialization it will allocate a memory map of `HEAP_SIZE` and align
    /// it to `BLOCK_SIZE`. During normal runtime it will allocate blocks on the
    /// fly from this memory map and store returned blocks in a list.
    ///
    /// Blocks from this `BlockAllocator` are always aligned to `BLOCK_SIZE`.
    ///
    /// The list of returned free blocks is a stack. The `BlockAllocator` will
    /// first exhaust the returned free blocks and then fall back to allocating
    /// new blocks from the memory map. This means it will return recently
    /// returned blocks first.
    pub struct BlockAllocator {
        /// A list of returned (free) blocks.
        pub(crate) free_blocks: Mutex<Vec<*mut BlockInfo>>,
        pub(crate) allocated: DashSet<*mut BlockInfo>,
        pub(crate) unavailable_blocks: Mutex<Vec<*mut BlockInfo>>,
        pub(crate) recyclable_blocks: Mutex<Vec<*mut BlockInfo>>,
    }
    impl BlockAllocator {
        pub fn new() -> Self {
            Self {
                free_blocks: Mutex::new(Vec::with_capacity(4)),
                allocated: DashSet::with_capacity(4),
                unavailable_blocks: Mutex::new(vec![]),
                recyclable_blocks: Mutex::new(vec![]),
            }
        }
        /// Get a new block aligned to `BLOCK_SIZE`.
        pub fn get_block(&self) -> *mut BlockInfo {
            let b = self.free_blocks.lock().pop();
            b.unwrap_or_else(|| self.build_next_block())
        }
        pub fn drain_blocks(&self) -> Vec<*mut BlockInfo> {
            let mut x = self.recyclable_blocks.lock().drain(..).collect::<Vec<_>>();
            let mut y = self.unavailable_blocks.lock().drain(..).collect::<Vec<_>>();
            x.drain(..).chain(y.drain(..)).collect()
        }
        /// Return a collection of blocks.
        pub fn return_blocks(&self, blocks: Vec<*mut BlockInfo>) {
            self.free_blocks.lock().extend(blocks);
        }

        /// Return the number of unallocated blocks.
        pub fn available_blocks(&self) -> usize {
            self.free_blocks.lock().len()
            //(((self.data_bound as usize) - (self.data as usize)) % BLOCK_SIZE) + self.free_blocks.len()
        }

        /// Return if an address is within the bounds of the memory map.
        pub fn is_in_space(&self, object: GCObjectRef) -> bool {
            unsafe {
                let block = BlockInfo::get_block_ptr(object);
                self.allocated.contains(&block) && { (&*block).is_in_block(object) }
            }
        }
        pub fn recycle(&self, blocks: Vec<*mut BlockInfo>) {
            self.recyclable_blocks.lock().extend(blocks);
        }
        pub fn recyclable_pop(&self) -> Option<*mut BlockInfo> {
            self.recyclable_blocks.lock().pop()
        }
        pub fn return_unavailable(&self, block: *mut BlockInfo) {
            self.unavailable_blocks.lock().push(block);
        }

        fn build_next_block(&self) -> *mut BlockInfo {
            unsafe {
                let block = alloc_zeroed(BLOCK_LAYOUT).cast::<BlockInfo>();
                std::ptr::write(block, BlockInfo::new());
                self.allocated.insert(block);
                block
            }
        }
    }
}
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod lockfree {
    use super::*;
    pub struct BlockAllocator {
        pub(crate) free_blocks: LockFreeList<*mut BlockInfo>,
        pub(crate) allocated: DashSet<*mut BlockInfo>,
        pub(crate) unavailable_blocks: LockFreeList<*mut BlockInfo>,
        pub(crate) recyclable_blocks: LockFreeList<*mut BlockInfo>,
        pub(crate) free_blocks_size: AtomicUsize,
        pub(crate) threshold: AtomicUsize,
    }
    impl BlockAllocator {
        pub fn new() -> Self {
            Self {
                free_blocks: LockFreeList::new(),
                allocated: DashSet::new(),
                unavailable_blocks: LockFreeList::new(),
                recyclable_blocks: LockFreeList::new(),
                free_blocks_size: AtomicUsize::new(0),
                threshold: AtomicUsize::new(2),
            }
        }

        pub fn drain_blocks(&self) -> Vec<*mut BlockInfo> {
            let mut blocks = vec![];
            while let Some(block) = self.recyclable_blocks.pop() {
                blocks.push(block);
            }
            while let Some(block) = self.unavailable_blocks.pop() {
                blocks.push(block);
            }
            blocks
        }
        /// Get a new block aligned to `BLOCK_SIZE`.
        pub fn get_block(&self) -> *mut BlockInfo {
            self.free_blocks
                .pop()
                .and_then(|b| {
                    self.free_blocks_size.fetch_sub(1, Ordering::AcqRel);
                    Some(b)
                })
                .unwrap_or_else(|| self.build_next_block())
        }

        /// Return a collection of blocks.
        pub fn return_blocks(&self, blocks: Vec<*mut BlockInfo>) {
            self.free_blocks_size
                .fetch_add(blocks.len(), Ordering::AcqRel);
            for block in blocks.iter() {
                self.free_blocks.push(*block);
            }
            //self.free_blocks.extend(blocks);
        }

        /// Return the number of unallocated blocks.
        pub fn available_blocks(&self) -> usize {
            self.free_blocks_size.load(Ordering::Acquire)
            //(((self.data_bound as usize) - (self.data as usize)) % BLOCK_SIZE) + self.free_blocks.len()
        }

        /// Return if an address is within the bounds of the memory map.
        pub fn is_in_space(&self, object: GCObjectRef) -> bool {
            unsafe {
                let block = BlockInfo::get_block_ptr(object);
                self.allocated.contains(&block) && { (&*block).is_in_block(object) }
            }
        }

        fn build_next_block(&self) -> *mut BlockInfo {
            unsafe {
                if self.allocated.len() >= self.threshold.load(Ordering::Relaxed) {
                    crate::VM.collect();
                }
                let block = alloc_zeroed(BLOCK_LAYOUT).cast::<BlockInfo>();
                std::ptr::write(block, BlockInfo::new());
                self.allocated.insert(block);
                block
            }
        }
        pub fn recycle(&self, blocks: Vec<*mut BlockInfo>) {
            for block in blocks.iter() {
                self.recyclable_blocks.push(*block);
            }
        }
        pub fn recyclable_pop(&self) -> Option<*mut BlockInfo> {
            self.recyclable_blocks.pop()
        }
        pub fn return_unavailable(&self, block: *mut BlockInfo) {
            self.unavailable_blocks.push(block);
        }
    }
}
pub use lockfree::BlockAllocator;
