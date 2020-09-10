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

pub struct BlockDirectory {
    blocks: Vec<*mut MarkedBlockHandle>,
    free_block_indicies: Vec<u32>,
    cell_size: usize,
    /// After you do something to a block based on one of these cursors, you clear the bit in the
    /// corresponding bitvector and leave the cursor where it was. We can use u32 instead of usize since
    /// this number is bound by capacity of Vec blocks, which must be within u32.
    empty_cursor: u32,
    unswept_cursor: u32,

    next_directory: *mut Self,
    local_allocators: LinkedList<LocalAllocator>,
    bits: BitVec,
}

impl BlockDirectory {
    pub fn find_empty_block_to_seal(&mut self) -> *mut MarkedBlockHandle {
        let empty_cursor = find_bit(&self.bits, self.empty_cursor as _, true);
        self.empty_cursor = empty_cursor as _;
        if empty_cursor >= self.blocks.len() {
            0 as *mut MarkedBlockHandle
        } else {
            self.blocks[empty_cursor]
        }
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
}
