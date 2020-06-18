use super::block_info::BlockInfo;
use crate::gc::{constants::*, GCObjectRef};
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::collections::HashSet;

#[cfg(not(feature="small-heap"))]
pub mod normal_heap {
    use memmap::MmapMut;
    use super::*;
    pub struct MemoryMap {
        mem: *mut u8,
        size: u32,
    }
    impl MemoryMap {
        pub fn new(size: usize) -> Self {
            let mmap = MmapMut::make_anon(size).unwrap().as_mut_ptr();
            Self {
                mem: mmap.cast()
            }
        }

        pub fn aligned(&self) -> *mut u8 {
            let offset = BLOCK_SIZE - (self.mem as usize) % BLOCK_SIZE;
            unsafe {
                self.mem.offset(offset as isize)
            }
        }
        pub fn start(&self) -> *mut u8 {
            self.mem
        }
        pub fn bound(&self) -> *mut u8 {
            unsafe {
                self.mem.offset(self.size)
            }
        }
        
    }

    impl Drop for MemoryMap {
        fn drop(&mut self) {
            unsafe {
                libc::free(self.mem as *mut _);
            }
        }
    }
    use std::sync::atomic::{AtomicUsize,Ordering};
    use parking_lot::Mutex;
    pub struct BlockAllocator {
        mmap: MemoryMap,
        data: AtomicUsize,
        data_bound: AtomicUsize,
        free_blocks: Vec<*mut BlockInfo>
    }


}

#[cfg(feature="small-heap")]
pub use small_heap_allocator::*;

#[cfg(feature="small-heap")]
pub mod small_heap_allocator {
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
    pub(crate) free_blocks: Vec<*mut BlockInfo>,
    allocated: HashSet<*mut BlockInfo>,
    pub(crate) unavailable_blocks: Vec<*mut BlockInfo>,
    pub(crate) recyclable_blocks: Vec<*mut BlockInfo>,
}
const BLOCK_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(BLOCK_SIZE, BLOCK_SIZE) };
impl BlockAllocator {
    pub fn new() -> Self {
        Self {
            free_blocks: Vec::with_capacity(4),
            allocated: HashSet::with_capacity(4),
            unavailable_blocks: vec![],
            recyclable_blocks: vec![],
        }
    }
    /// Get a new block aligned to `BLOCK_SIZE`.
    pub fn get_block(&mut self) -> *mut BlockInfo {
        self.free_blocks
            .pop()
            .unwrap_or_else(|| self.build_next_block())
    }

    /// Return a collection of blocks.
    pub fn return_blocks(&mut self, blocks: Vec<*mut BlockInfo>) {
        self.free_blocks.extend(blocks);
    }

    /// Return the number of unallocated blocks.
    pub fn available_blocks(&self) -> usize {
        self.free_blocks.len()
        //(((self.data_bound as usize) - (self.data as usize)) % BLOCK_SIZE) + self.free_blocks.len()
    }

    /// Return if an address is within the bounds of the memory map.
    pub fn is_in_space(&self, object: GCObjectRef) -> bool {
        unsafe {
            let block = BlockInfo::get_block_ptr(object);
            self.allocated.contains(&block) && { (&*block).is_in_block(object) }
        }
    }

    fn build_next_block(&mut self) -> *mut BlockInfo {
        unsafe {
            let block = alloc_zeroed(BLOCK_LAYOUT).cast::<BlockInfo>();
            std::ptr::write(block, BlockInfo::new());
            self.allocated.insert(block);
            block
        }
    }
}
}
