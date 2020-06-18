use super::*;
use crate::gc::immix_space::block_allocator::BlockAllocator;
use crate::gc::immix_space::block_info::BlockInfo;

use normal_allocator::*;
use overflow_allocator::*;
use parking_lot::RwLock;
use std::sync::Arc;
pub struct LocalAllocator {
    #[allow(dead_code)]
    id: usize,
    normal_allocator: NormalAllocator,
    overflow_allocator: OverflowAllocator,
    /// The current block to allocate from.
    current_block: Option<BlockTuple>,
}

impl LocalAllocator {
    pub fn new(id: usize, block_allocator: Arc<RwLock<BlockAllocator>>) -> Self {
        Self {
            id,
            current_block: None,
            normal_allocator: NormalAllocator::new(block_allocator.clone()),
            overflow_allocator: OverflowAllocator::new(block_allocator),
        }
    }
    pub fn init(&mut self, collect: bool) {}
}

impl Allocator for LocalAllocator {
    fn get_all_blocks(&mut self) -> Vec<*mut BlockInfo> {
        self.normal_allocator
            .get_all_blocks()
            .drain(..)
            .chain(self.overflow_allocator.get_all_blocks().drain(..))
            .collect()
    }

    fn take_current_block(&mut self) -> Option<BlockTuple> {
        self.normal_allocator.take_current_block();
        self.overflow_allocator.take_current_block();
        self.current_block.take()
    }

    fn put_current_block(&mut self, block_tuple: BlockTuple) {
        self.normal_allocator.put_current_block(block_tuple);
        self.overflow_allocator.put_current_block(block_tuple);
        self.current_block = Some(block_tuple);
    }

    fn get_new_block(&mut self) -> Option<BlockTuple> {
        log::debug!("Request new block");
        let b = self.normal_allocator.block_allocator.write().get_block();
        unsafe {
            (*b).set_allocated();
        }
        Some((b, LINE_SIZE as u16, (BLOCK_SIZE - 1) as u16))
    }

    fn handle_full_block(&mut self, block: *mut BlockInfo) {
        self.normal_allocator
            .block_allocator
            .write()
            .unavailable_blocks
            .push(block);
    }
    fn handle_no_hole(&mut self, size: usize) -> Option<BlockTuple> {
        if size >= LINE_SIZE {
            None
        } else {
            let r = self
                .normal_allocator
                .block_allocator
                .write()
                .recyclable_blocks
                .pop();
            match r {
                None => None,
                Some(block) => match unsafe { (*block).scan_block((LINE_SIZE - 1) as u16) } {
                    None => {
                        self.handle_full_block(block);
                        self.handle_no_hole(size)
                    }
                    Some((low, high)) => self
                        .scan_for_hole(size, (block, low, high))
                        .or_else(|| self.handle_no_hole(size)),
                },
            }
        }
    }

    fn allocate(&mut self, size: usize) -> Option<GCObjectRef> {
        if size >= MEDIUM_OBJECT {
            self.overflow_allocator.allocate(size)
        } else {
            self.normal_allocator.allocate(size)
        }
    }
}

unsafe impl Send for LocalAllocator {}
unsafe impl Sync for LocalAllocator {}
