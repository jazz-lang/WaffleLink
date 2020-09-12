use super::local_allocator::*;
use super::*;
use bit_vec::BitVec;
use markedblock::*;
use std::collections::LinkedList;

fn find_bit(bv: &BitVec, x: usize, bit: bool) -> usize {
    for ix in x..bv.len() {
        if bv.get(ix) == Some(bit) {
            return ix;
        }
    }

    bv.len()
}
use parking_lot::Mutex;
pub struct BlockDirectory {
    pub blocks: Vec<*mut MarkedBlockHandle>,
    free_block_indicies: Vec<u32>,
    cell_size: usize,
    /// After you do something to a block based on one of these cursors, you clear the bit in the
    /// corresponding bitvector and leave the cursor where it was. We can use u32 instead of usize since
    /// this number is bound by capacity of Vec blocks, which must be within u32.
    empty_cursor: u32,
    unswept_cursor: u32,
    bitvector_lock: Mutex<()>,
    local_allocators_lock: Mutex<()>,
    next_directory: *mut Self,
    local_allocators: LinkedList<LocalAllocator>,
    bits: block_directory_bits::BlockDirectoryBits,
}
use block_directory_bits::*;
impl BlockDirectory {
    pub fn cell_size(&self) -> usize {
        self.cell_size
    }
    pub fn find_empty_block_to_seal(&mut self) -> *mut MarkedBlockHandle {
        /*self.blocks
        .iter()
        .find(|x| unsafe { (&***x).empty })
        .copied()
        .unwrap_or(0 as *mut _)*/
        self.empty_cursor = self
            .bits
            .empty()
            .base
            .find_bit(self.empty_cursor as _, true) as u32;
        if self.empty_cursor >= self.blocks.len() as u32 {
            return core::ptr::null_mut();
        }
        self.blocks[self.empty_cursor as usize]
    }

    pub fn find_block_for_allocation(
        &mut self,
        allocator: &mut LocalAllocator,
    ) -> *mut MarkedBlockHandle {
        loop {
            let res = self.blocks.iter().enumerate().find(|(ix, block)| unsafe {
                if (&***block).empty || (&***block).can_allocate {
                    true
                } else {
                    false
                }
            });
            if res.is_none() {
                return 0 as *mut _;
            }
            let res = res.unwrap();
            allocator.alloc_cursor = res.0 as _;
            let block = unsafe { &mut **res.1 };
            block.empty = false;
            block.can_allocate = true;
            return *res.1;
        }
    }

    pub fn find_blocks_for_allocation(
        &mut self,
        local: &mut LocalAllocator,
        mut and_then: impl FnMut(&mut LocalAllocator, *mut MarkedBlockHandle) -> *mut (),
    ) -> *mut () {
        for block in self.blocks.iter().filter(|block| unsafe {
            (&***block).empty || ((&***block).can_allocate && !(&***block).empty)
        }) {
            let res = and_then(local, *block);
            if res.is_null() {
                continue;
            }
            return res;
        }
        0 as *mut ()
    }
    pub fn find_blocks_to_seal(&mut self, mut and_then: impl FnMut(*mut MarkedBlockHandle)) {
        self.blocks
            .iter()
            .filter(|block| unsafe { (&***block).empty })
            .for_each(|block| {
                and_then(*block);
            })
    }
    pub fn remove_block(&mut self, p: &mut MarkedBlockHandle) {
        self.blocks[p.index as usize] = 0 as *mut _;
        self.free_block_indicies.push(p.index);
        p.did_remove_from_directory();
    }
    pub fn create_block(&mut self) -> &'static mut MarkedBlockHandle {
        let handle = MarkedBlock::create();
        handle
    }
    pub fn add_block(&mut self, p: *mut MarkedBlockHandle) {
        unsafe {
            let mut handle = &mut *p;
            let mut index;
            if let Some(ix) = self.free_block_indicies.pop() {
                self.blocks[ix as usize] = p;
                index = ix;
            } else {
                index = self.blocks.len() as u32;
                self.blocks.push(p);
            }

            // This is the point at which the block learns of its cell_size()
            handle.did_add_to_directory(self as *mut _, index);
            handle.empty = true;
        }
    }
}
