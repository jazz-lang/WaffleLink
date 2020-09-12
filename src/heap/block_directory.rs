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
macro_rules! for_each_bit_vector {
    ($self: expr,$bits: ident,$f: expr) => {
        let this = $self;

        let mut $bits = $self.bits.live();
        $f;
        let mut $bits = ($self.bits.empty());
        $f;
        let mut $bits = ($self.bits.allocated());
        $f;
        let mut $bits = ($self.bits.can_allocate_but_not_empty());
        $f;
        let mut $bits = ($self.bits.destructible());
        $f;
        let mut $bits = ($self.bits.eden());
        $f;
        let mut $bits = ($self.bits.unswept());
        $f;
        let mut $bits = ($self.bits.marking_not_empty());
        $f;
        let mut $bits = ($self.bits.marking_retired());
        $f;
    };
}

pub struct BlockDirectory {
    pub blocks: Vec<*mut MarkedBlockHandle>,
    pub free_block_indicies: Vec<u32>,
    pub cell_size: usize,
    /// After you do something to a block based on one of these cursors, you clear the bit in the
    /// corresponding bitvector and leave the cursor where it was. We can use u32 instead of usize since
    /// this number is bound by capacity of Vec blocks, which must be within u32.
    pub empty_cursor: u32,
    pub unswept_cursor: u32,
    pub bitvector_lock: Mutex<()>,
    pub local_allocators_lock: Mutex<()>,
    pub next_directory: *mut Self,
    pub local_allocators: LinkedList<LocalAllocator>,
    pub bits: block_directory_bits::BlockDirectoryBits,
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
            allocator.alloc_cursor = (self
                .bits
                .can_allocate_but_not_empty()
                .or(&*self.bits.empty()))
            .find_bit(allocator.alloc_cursor as _, true) as _;
            if allocator.alloc_cursor >= self.blocks.len() as u32 {
                return 0 as *mut _;
            }
            let block_index = allocator.alloc_cursor;
            allocator.alloc_cursor += 1;
            let result = self.blocks[block_index as usize];
            self.bits
                .set_is_can_allocate_but_not_empty(block_index as _, true);
            return result;
        }
    }
    pub fn resume_allocating(&mut self) {
        self.local_allocators.iter_mut().for_each(|local| {
            local.resume_allocating();
        });
    }

    pub fn stop_allocating_for_good(&mut self) {
        self.local_allocators.iter_mut().for_each(|local| {
            local.stop_allocating_for_good();
        });
        let lock = self.local_allocators_lock.lock();
        self.local_allocators.clear();
        drop(lock);
    }
    pub fn stop_allocating(&mut self) {
        self.local_allocators.iter_mut().for_each(|local| {
            local.stop_allocating_for_good();
        });
    }

    pub fn prepare_for_allocation(&mut self) {
        self.local_allocators
            .iter_mut()
            .for_each(|local| local.reset());
        self.unswept_cursor = 0;
        self.empty_cursor = 0;
        self.bits.eden().clear_all();
    }
    pub fn remove_block(&mut self, p: &mut MarkedBlockHandle) {
        self.blocks[p.index as usize] = 0 as *mut _;
        self.free_block_indicies.push(p.index);
        let lock = self.bitvector_lock.lock();
        for_each_bit_vector!(&self, bits, {
            bits.set_at(p.index as usize, false);
        });
        drop(lock);
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
                let old_cap = self.blocks.capacity();
                self.blocks.push(p);
                if self.blocks.capacity() != old_cap {
                    let lock = self.bitvector_lock.lock();
                    self.bits.resize(self.blocks.capacity());
                }
            }

            // This is the point at which the block learns of its cell_size()
            handle.did_add_to_directory(self as *mut _, index);
            handle.empty = true;
            self.bits.empty().set_at(index as _, true);
            self.bits.live().set_at(index as _, true);
        }
    }
}
