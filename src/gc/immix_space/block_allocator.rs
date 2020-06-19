use super::block_info::BlockInfo;
use crate::gc::{constants::*, GCObjectRef};
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::collections::HashSet;

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
    /*pub(crate) free_blocks: Vec<*mut BlockInfo>,
    allocated: HashSet<*mut BlockInfo>,
    pub(crate) unavailable_blocks: Vec<*mut BlockInfo>,
    pub(crate) recyclable_blocks: Vec<*mut BlockInfo>,*/
    chunks: Mutex<Vec<*mut Chunk>>,
    current: AtomicPtr<Chunk>,
    available_blocks: AtomicUsize,
}
static AVAIL_CH_BLOCKS: AtomicUsize = AtomicUsize::new(0);

const BLOCK_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(BLOCK_SIZE, BLOCK_SIZE) };
impl BlockAllocator {
    pub fn new() -> Self {
        let chunk = Chunk::new();
        Self {
            chunks: Mutex::new(vec![chunk]),
            current: AtomicPtr::new(chunk),
            available_blocks: AtomicUsize::new(0),
        }
    }

    pub fn get_recyclable_block(&self) -> Option<*mut BlockInfo> {
        let c = self.current.load(Ordering::Acquire);
        unsafe {
            (&*c).recyclable.pop().or_else(|| {
                let lock = self.chunks.lock();
                for ch in lock.iter() {
                    if let Some(block) = (&**ch).recyclable.pop() {
                        return Some(block);
                    }
                }
                None
            })
        }
    }
    /// Get a new block aligned to `BLOCK_SIZE`.
    pub fn get_block(&self) -> *mut BlockInfo {
        let c = self.current.load(Ordering::Acquire);
        unsafe {
            (&*c).free.pop().unwrap_or_else(|| {
                let block = self.build_next_block();
                log::trace!("Alloc block");
                block.write(BlockInfo::new());
                block
            })
        }
    }

    /// Return a collection of blocks.
    pub fn return_blocks(&self, blocks: Vec<*mut BlockInfo>) {
        unsafe {
            for block in blocks.iter() {
                let chunk = ((*block) as usize & !(CHUNK_SIZE - 1)) as *mut Chunk;
                (&*chunk).free.push(*block);
            }
        }
        //self.free_blocks.extend(blocks);
    }
    pub fn get_all_blocks(&self) -> Vec<*mut BlockInfo> {
        let mut blocks = vec![];
        for ch in self.chunks.lock().iter() {
            unsafe {
                let ch = &mut **ch;
                while let Some(block) = ch.recyclable.pop() {
                    blocks.push(block);
                }
                while let Some(block) = ch.unavail.pop() {
                    blocks.push(block);
                }
            }
        }
        blocks
    }
    pub fn return_recyclable_blocks(&self, blocks: Vec<*mut BlockInfo>) {
        unsafe {
            for block in blocks.iter() {
                let chunk = ((*block) as usize & !(CHUNK_SIZE - 1)) as *mut Chunk;
                (&*chunk).recyclable.push(*block);
            }
        }
        //self.free_blocks.extend(blocks);
    }

    pub fn return_unavailable(&self, block: *mut BlockInfo) {
        let chunk = (block as usize & !(CHUNK_SIZE - 1)) as *mut Chunk;
        unsafe {
            (&*chunk).unavail.push(block);
        }
    }

    /// Return the number of unallocated blocks.
    pub fn available_blocks(&self) -> usize {
        self.available_blocks.load(Ordering::Relaxed)
            + (AVAIL_CH_BLOCKS.load(Ordering::Relaxed) % BLOCK_SIZE)
        //self.free_blocks.len()
        //(((self.data_bound as usize) - (self.data as usize)) % BLOCK_SIZE) + self.free_blocks.len()
    }

    /// Return if an address is within the bounds of the memory map.
    pub fn is_in_space(&self, object: GCObjectRef) -> bool {
        unsafe {
            let c = self.current.load(Ordering::Acquire);
            let ptr = object.raw() as usize;
            let bound = (&*c).bound();
            let start = (&*c).start();
            if ptr >= start && ptr < bound {
                return true;
            } else {
                let lock = self.chunks.lock();
                for ch in lock.iter() {
                    let bound = (&**ch).bound();
                    let start = (&**ch).start();
                    if ptr >= start && ptr < bound {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn build_next_block(&self) -> *mut BlockInfo {
        unsafe {
            let c = self.current.load(Ordering::Acquire);
            (&*c)
                .build_next_block()
                .or_else(|| {
                    let lock = self.chunks.lock();
                    for ch in lock.iter() {
                        if let Some(block) = (&**ch).build_next_block() {
                            self.current
                                .store(&**ch as *const Chunk as *mut Chunk, Ordering::SeqCst);
                            return Some(block);
                        }
                    }
                    None
                })
                .unwrap_or_else(|| {
                    let chunk = Chunk::new();
                    let block = (&*chunk).build_next_block().unwrap();
                    self.current.store(chunk, Ordering::SeqCst);
                    self.chunks.lock().push(chunk);
                    block
                })
        }
    }
}

pub const CHUNK_SIZE: usize = super::G * 2;
use crate::util::lflist::LockFreeList;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
pub struct Chunk {
    pub start: AtomicUsize,
    pub cursor: AtomicUsize,
    pub recyclable: LockFreeList<*mut BlockInfo>,
    pub free: LockFreeList<*mut BlockInfo>,
    pub unavail: LockFreeList<*mut BlockInfo>,
}
use crate::gc::Address;
use crate::util::mem::*;
impl Chunk {
    pub fn new() -> *mut Self {
        unsafe {
            let mem = Address::from_ptr(alloc_zeroed(Layout::from_size_align_unchecked(
                CHUNK_SIZE, CHUNK_SIZE,
            )));
            std::ptr::write_bytes(mem.to_mut_ptr::<u8>(), 0, CHUNK_SIZE);
            let offset = BLOCK_SIZE - mem.to_usize() % BLOCK_SIZE;
            let aligned = mem.offset(std::mem::size_of::<Self>() + offset).to_usize();
            log::debug!("Chunk cursor set to {:p}", aligned as *mut u8);
            AVAIL_CH_BLOCKS.fetch_add(
                mem.offset(CHUNK_SIZE).to_usize() - aligned,
                Ordering::AcqRel,
            );
            mem.to_mut_ptr::<Self>().write(Self {
                start: AtomicUsize::new(mem.to_usize() + std::mem::size_of::<Self>()),
                cursor: AtomicUsize::new(aligned),
                free: LockFreeList::new(),
                recyclable: LockFreeList::new(),
                unavail: LockFreeList::new(),
            });
            mem.to_mut_ptr()
        }
    }
    pub fn start(&self) -> usize {
        self.start.load(Ordering::Relaxed)
    }

    pub fn bound(&self) -> usize {
        self.start() + CHUNK_SIZE
    }

    pub fn build_next_block(&self) -> Option<*mut BlockInfo> {
        let mut old = self.cursor.load(Ordering::Relaxed);
        let mut new;
        loop {
            new = old + BLOCK_SIZE;
            if new > self.bound() {
                return None;
            }

            let res =
                self.cursor
                    .compare_exchange_weak(old, new, Ordering::SeqCst, Ordering::Relaxed);
            match res {
                Ok(_) => break,
                Err(x) => old = x,
            }
        }
        AVAIL_CH_BLOCKS.fetch_sub(BLOCK_SIZE, Ordering::Relaxed);
        debug_assert!(old >= self.start() && old < self.bound());
        Some(old as *mut _)
    }

    pub fn from_pointer(ptr: usize) -> *mut Self {
        let a = ptr & !(CHUNK_SIZE - 1);
        a as *mut Self
    }
}
