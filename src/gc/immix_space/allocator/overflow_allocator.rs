use super::*;
use crate::gc::immix_space::block_allocator::BlockAllocator;
use crate::gc::immix_space::block_info::BlockInfo;
use parking_lot::RwLock;
use std::sync::Arc;

/// The `OverflowAllocator` is used to allocate *medium* sized objects
/// (objects of at least `MEDIUM_OBJECT` bytes size) within the immix space to
/// limit fragmentation in the `NormalAllocator`.
pub struct OverflowAllocator {
    /// The global `BlockAllocator` to get new blocks from.
    block_allocator: Arc<RwLock<BlockAllocator>>,

    /// The exhausted blocks.
    unavailable_blocks: Vec<*mut BlockInfo>,

    /// The current block to allocate from.
    current_block: Option<BlockTuple>,
}
impl OverflowAllocator {
    /// Create a new `OverflowAllocator` backed by the given `BlockAllocator`.
    pub fn new(block_allocator: Arc<RwLock<BlockAllocator>>) -> OverflowAllocator {
        OverflowAllocator {
            block_allocator: block_allocator,
            unavailable_blocks: Vec::new(),
            current_block: None,
        }
    }
}

impl Allocator for OverflowAllocator {
    fn get_all_blocks(&mut self) -> Vec<*mut BlockInfo> {
        self.unavailable_blocks
            .drain(..)
            .chain(self.current_block.take().map(|b| b.0))
            .collect()
    }

    fn take_current_block(&mut self) -> Option<BlockTuple> {
        self.current_block.take()
    }

    fn put_current_block(&mut self, block_tuple: BlockTuple) {
        self.current_block = Some(block_tuple);
    }

    fn get_new_block(&mut self) -> Option<BlockTuple> {
        log::debug!("Request new block");
        let b = self.block_allocator.write().get_block();
        /*.map(|b| unsafe {
            (*b).set_allocated();
            b
        })
        .map(|block| (block, LINE_SIZE as u16, (BLOCK_SIZE - 1) as u16))*/
        unsafe {
            (*b).set_allocated();
        }
        Some((b, LINE_SIZE as u16, (BLOCK_SIZE - 1) as u16))
    }

    #[allow(unused_variables)]
    fn handle_no_hole(&mut self, size: usize) -> Option<BlockTuple> {
        None
    }

    fn handle_full_block(&mut self, block: *mut BlockInfo) {
        log::debug!("Push block {:p} into unavailable_blocks", block);
        self.unavailable_blocks.push(block);
    }
}
