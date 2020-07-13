use super::*;
use crate::gc::Address;
use crate::object::*;
use bitset_core::BitSet;
use std::collections::HashSet;
pub struct HeapBlock {
    cell_size: usize,
    free_list: *mut FreeListEntry,
    bitset: HashSet<Address>,
    cursor: Address,
    storage: u8,
}

impl HeapBlock {
    pub fn sweep(&mut self) -> bool {
        let mut all_free = true;
        let mut free_list: *mut FreeListEntry = std::ptr::null_mut();
        self.for_each_cell_mut(|this, cell_addr| unsafe {
            if this.is_marked(cell_addr) {
                let mut cell = Ref {
                    ptr: cell_addr.to_mut_ptr::<Obj>(),
                };
                if cell.header().is_marked_non_atomic() {
                    cell.header_mut().unmark_non_atomic();
                    all_free = false;
                } else {
                    this.unmark(cell_addr);
                    if let Some(destroy_fn) = cell.header().vtbl().destroy_fn {
                        destroy_fn(cell);
                    }
                    std::ptr::write_bytes(cell_addr.to_mut_ptr::<u8>(), 0, this.cell_size);
                    if free_list.is_null() {
                        free_list = cell.raw() as *mut _;
                        (&mut *free_list).next = std::ptr::null_mut();
                    } else {
                        let next = free_list;
                        free_list = cell.raw() as *mut _;
                        (&mut *free_list).next = next as *mut _;
                    }
                }
            } else {
                let next = free_list;
                free_list = cell_addr.to_mut_ptr();
                (&mut *free_list).next = next as *mut _;
            }
        });
        self.free_list = free_list;
        true
    }
    pub fn allocate(&mut self) -> Address {
        let addr = if self.cursor.offset(self.cell_size) < self.storage().offset(Self::BLOCK_SIZE) {
            let c = self.cursor;
            self.cursor = self.cursor.offset(self.cell_size);
            c
        } else {
            if self.free_list.is_null() {
                return Address::null();
            } else {
                unsafe {
                    let x = self.free_list;
                    self.free_list = (&*x).next.cast();
                    Address::from_ptr(x)
                }
            }
        };
        if addr.is_non_null() {
            self.mark(addr);
        }
        addr
    }
    pub fn new(cell_size: usize) -> Box<Self> {
        let mem = unsafe {
            std::alloc::alloc_zeroed(std::alloc::Layout::from_size_align_unchecked(
                16 * 1024,
                16 * 1024,
            ))
            .cast::<Self>()
        };

        unsafe {
            mem.write(Self {
                cell_size,
                free_list: std::ptr::null_mut(),
                bitset: HashSet::new(),
                cursor: Address::null(),
                storage: 0,
            });
            let mut this = Box::from_raw(mem);
            this.cursor = Address::from_ptr(&this.storage);
            this.init_bitset();
            this
        }
    }
    pub const BLOCK_SIZE: usize = 16 * 1024;
    fn init_bitset(&mut self) {
        /*let count = self.cell_count();
        for _ in 0..(count / 64) {
            self.bitset.push(0);
        }
        self.bitset.bit_init(false);*/
    }
    pub fn cell_bit(&self, cell_addr: Address) -> usize {
        cell_addr.to_usize() % Self::BLOCK_SIZE
        // (cell_addr.to_usize() - self as *const Self as usize) / self.cell_size()
    }
    pub fn mark(&mut self, cell: Address) {
        self.bitset.insert(cell);
        //self.bitset.bit_set(Self::cell_bit(self, cell));
    }

    pub fn unmark(&mut self, cell: Address) {
        self.bitset.remove(&cell);
        //self.bitset.bit_reset(Self::cell_bit(self, cell));
    }

    pub fn is_marked(&self, cell: Address) -> bool {
        self.bitset.contains(&cell)
        //self.bitset.bit_test(Self::cell_bit(self, cell))
    }
    pub fn cell_size(&self) -> usize {
        self.cell_size
    }
    pub fn cell_count(&self) -> usize {
        return (Self::BLOCK_SIZE - std::mem::size_of::<Self>() - 1) / self.cell_size;
    }

    pub fn for_each_cell(&self, mut callback: impl FnMut(Address)) {
        for i in 0..self.cell_count() {
            callback(self.cell(i));
        }
    }
    pub fn for_each_cell_mut(&mut self, mut callback: impl FnMut(&mut Self, Address)) {
        for i in 0..self.cell_count() {
            callback(self, self.cell(i));
        }
    }
    pub fn storage(&self) -> Address {
        Address::from_ptr(&self.storage)
    }
    pub fn cell(&self, x: usize) -> Address {
        return self.storage().offset(x * self.cell_size());
    }
}

pub struct FreeListEntry {
    next: *mut u8,
}
