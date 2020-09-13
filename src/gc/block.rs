pub const BLOCK_SIZE: usize = 16 * 1024;
pub const ATOM_SIZE: usize = 16;
pub const ATOMS_PER_BLOCK: usize = BLOCK_SIZE / ATOM_SIZE;
pub const MAX_NUMBER_OF_LOWER_TIER_CELLS: usize = 8;
pub const END_ATOM: usize = (BLOCK_SIZE - core::mem::size_of::<BlockHeader>()) / ATOM_SIZE;
pub const PAYLOAD_SIZE: usize = END_ATOM * ATOM_SIZE;
pub const FOOTER_SIZE: usize = BLOCK_SIZE - PAYLOAD_SIZE;
pub const ATOM_ALIGNMENT_MASK: usize = ATOM_SIZE - 1;
const_assert!(
    PAYLOAD_SIZE == (BLOCK_SIZE - core::mem::size_of::<BlockHeader>()) & !(ATOM_SIZE - 1)
);

use crate::gc::*;

#[repr(C)]
pub struct FreeCell {
    bytes: u64,
    next: *mut Self,
}

pub struct FreeList {
    head: *mut FreeCell,
}

impl FreeList {
    pub const fn new() -> Self {
        Self {
            head: core::ptr::null_mut(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub fn allocate(&mut self) -> Address {
        if self.is_empty() {
            return Address::null();
        }
        unsafe {
            let prev = self.head;
            self.head = (&*prev).next;
            Address::from_ptr(prev)
        }
    }

    pub fn free(&mut self, cell: Address) {
        unsafe {
            let cell = cell.to_mut_ptr::<FreeCell>();
            (&mut *cell).next = self.head;
            self.head = cell;
        }
    }
}

pub type Atom = [u8; ATOM_SIZE];

pub struct BlockHeader {
    cell_size: u32,
    freelist: FreeList,
    /// If this set to true then we do not try to allocate from this block.
    can_allocate: bool,
    /// If true we didn't sweep this block
    unswept: bool,
    bitmap: bitmap::BitMap,
    block: *mut Block,
}

use std::alloc::{alloc, dealloc, Layout};

pub struct Block {}

impl Block {
    pub fn new(cell_size: usize) -> &'static mut BlockHeader {
        unsafe {
            let memory =
                alloc(Layout::from_size_align_unchecked(BLOCK_SIZE, BLOCK_SIZE)).cast::<Self>();
            ((&*memory).header() as *mut BlockHeader).write(BlockHeader {
                unswept: false,
                can_allocate: true,
                cell_size: cell_size as _,
                bitmap: bitmap::BitMap::new(),
                freelist: FreeList::new(),
                block: memory,
            });
            (&*memory).header().for_each_cell(|cell| {
                cell.to_mut_obj().zap(1);
                (&mut *memory).header().freelist.free(cell);
            });
            println!("{:p}", &(*memory).header());
            (&*memory).header()
        }
    }

    pub fn header(&self) -> &mut BlockHeader {
        unsafe { &mut *self.atoms().offset(END_ATOM as _).cast() }
    }
    pub fn atom_number(&self, p: Address) -> u32 {
        let atom_n = self.candidate_atom_number(p);
        atom_n as _
    }

    pub fn candidate_atom_number(&self, p: Address) -> usize {
        return (p.to_usize() - self as *const Self as usize) / ATOM_SIZE;
    }

    pub fn atoms(&self) -> *mut Atom {
        self as *const Self as *mut Atom
    }
    pub const fn is_atom_aligned(p: Address) -> bool {
        (p.to_usize() & ATOM_ALIGNMENT_MASK) == 0
    }
}

impl BlockHeader {
    pub const fn atoms_per_cell(&self) -> usize {
        ((self.cell_size as usize + ATOM_SIZE - 1) / ATOM_SIZE) as _
    }
    pub const fn end_atom(&self) -> usize {
        END_ATOM - self.atoms_per_cell() + 1
    }
    pub const fn cell_size(&self) -> u32 {
        self.cell_size
    }

    pub fn begin(&self) -> Address {
        Address::from_ptr(self.block)
    }

    pub fn end(&self) -> Address {
        Address::from_ptr(self).offset(BLOCK_SIZE)
    }

    pub fn for_each_cell(&self, mut func: impl FnMut(Address)) {
        /*let mut i = self.cell_count();
        while i > 0 {
            func(self.cell(i as _));
            i -= 1;
        }*/
        let mut i = 0;
        while i < self.end_atom() {
            let cell = unsafe { self.atoms().offset(i as _) };
            func(Address::from_ptr(cell));
            i += self.atoms_per_cell();
        }
    }

    pub fn cell(&self, index: usize) -> Address {
        self.begin().offset(index * self.cell_size() as usize)
    }

    pub const fn cell_count(&self) -> u32 {
        (BLOCK_SIZE as u32 - core::mem::size_of::<Self>() as u32) / self.cell_size()
    }

    pub fn allocate(&mut self) -> Address {
        let addr = self.freelist.allocate();
        if addr.is_null() {
            self.can_allocate = false;
        }
        addr
    }
    pub fn sweep(&mut self) {
        let mut freelist = FreeList::new();
        self.for_each_cell(|cell| {
            let object = cell.to_mut_obj();
            if !self.is_marked(cell) {
                if !object.is_zapped() {
                    unsafe {
                        core::ptr::drop_in_place(object.trait_object());
                    }
                    object.zap(1);
                }
                freelist.free(cell);
            } else {
                debug_assert!(self.is_marked(cell));
            }
        });
        self.freelist = freelist;
    }

    pub fn test_and_set_marked(&self, p: Address) -> bool {
        self.bitmap
            .concurrent_test_and_set(self.atom_number(p) as _)
    }

    pub fn is_marked(&self, p: Address) -> bool {
        self.bitmap.get(self.atom_number(p) as _)
    }

    pub fn atom_number(&self, p: Address) -> u32 {
        let atom_n = self.candidate_atom_number(p);
        atom_n as _
    }

    pub fn candidate_atom_number(&self, p: Address) -> usize {
        return (p.to_usize() - self.begin().to_usize()) / ATOM_SIZE;
    }

    pub fn atoms(&self) -> *mut Atom {
        self.begin().to_mut_ptr()
    }
}
