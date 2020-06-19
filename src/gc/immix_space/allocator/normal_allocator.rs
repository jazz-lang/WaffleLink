use super::*;
use crate::gc::immix_space::block_allocator::BlockAllocator;
use crate::gc::immix_space::block_info::BlockInfo;
use parking_lot::RwLock;
use std::sync::Arc;
/// The `NormalAllocator` is the standard allocator to allocate objects within
/// the immix space.
///
/// Objects smaller than `MEDIUM_OBJECT` bytes are
pub struct NormalAllocator {
    /// The global `BlockAllocator` to get new blocks from.
    pub(super) block_allocator: Arc<BlockAllocator>,

    /// The current block to allocate from.
    current_block: Option<BlockTuple>,
}

impl NormalAllocator {
    /// Create a new `NormalAllocator` backed by the given `BlockAllocator`.
    pub fn new(block_allocator: Arc<BlockAllocator>) -> NormalAllocator {
        NormalAllocator {
            block_allocator: block_allocator,

            current_block: None,
        }
    }
}

impl Allocator for NormalAllocator {
    fn get_all_blocks(&mut self) -> Vec<*mut BlockInfo> {
        vec![]
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
        let b = self.block_allocator.get_block();
        /*.map(|b| unsafe {
            (*b).set_allocated();
            b
        })
        .map(|block| (block, LINE_SIZE as u16, (BLOCK_SIZE - 1) as u16))*/
        unsafe {
            log::debug!("Set allocated {:p}", b);
            (*b).set_allocated();
        }
        Some((b, LINE_SIZE as u16, (BLOCK_SIZE - 1) as u16))
    }

    fn handle_no_hole(&mut self, size: usize) -> Option<BlockTuple> {
        if size >= LINE_SIZE {
            None
        } else {
            let r = self.block_allocator.get_recyclable_block();
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

    fn handle_full_block(&mut self, block: *mut BlockInfo) {
        log::debug!("Push block {:p} into unavailable_blocks", block);
        self.block_allocator.return_unavailable(block);
    }
}
