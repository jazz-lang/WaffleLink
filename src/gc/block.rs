/// Block size must be at least as large as the system page size.
pub const BLOCK_SIZE: usize = 16 * 1024;
/// Single atom size
pub const ATOM_SIZE: usize = 16;
/// Numbers of atoms per block
pub const ATOMS_PER_BLOCK: usize = BLOCK_SIZE / ATOM_SIZE;
/// Lower tiers maximum
pub const MAX_NUMBER_OF_LOWER_TIER_CELLS: usize = 8;
/// End atom offset
pub const END_ATOM: usize = (BLOCK_SIZE - core::mem::size_of::<BlockHeader>()) / ATOM_SIZE;
/// Block payload size
pub const PAYLOAD_SIZE: usize = END_ATOM * ATOM_SIZE;
/// Block header size
pub const FOOTER_SIZE: usize = BLOCK_SIZE - PAYLOAD_SIZE;
/// Atom alignment mask
pub const ATOM_ALIGNMENT_MASK: usize = ATOM_SIZE - 1;
const_assert!(
    PAYLOAD_SIZE == (BLOCK_SIZE - core::mem::size_of::<BlockHeader>()) & !(ATOM_SIZE - 1)
);

use crate::gc::*;

/// Single freelist cell.
#[repr(C)]
pub struct FreeCell {
    bytes: u64,
    next: *mut Self,
}

/// Singly linked list used as free-list
pub struct FreeList {
    head: *mut FreeCell,
}

impl FreeList {
    /// Create new freelist
    pub const fn new() -> Self {
        Self {
            head: core::ptr::null_mut(),
        }
    }
    /// Is this freelist empty?
    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }
    /// Try to pop from list
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
    /// Push cell to list.
    pub fn free(&mut self, cell: Address) {
        unsafe {
            let cell = cell.to_mut_ptr::<FreeCell>();
            (&mut *cell).next = self.head;
            self.head = cell;
        }
    }
}

/// Atom representation
pub type Atom = [u8; ATOM_SIZE];

/// Heap allocated block header
pub struct BlockHeader {
    cell_size: u32,
    /// Free list for allocation
    pub freelist: FreeList,
    /// If this set to false then we do not try to allocate from this block.
    pub can_allocate: bool,
    /// If true we didn't sweep this block
    pub unswept: bool,
    /// Mark bitmap
    pub bitmap: bitmap::BitMap,
    /// Pointer to block.
    pub block: *mut Block,
}

use std::alloc::{alloc, dealloc, Layout};

/// Block is a page-aligned container for heap-allocated objects.
/// Objects are allocated within cells of the marked block. For a given
/// marked block, all cells have the same size. Objects smaller than the
/// cell size may be allocated in the marked block, in which case the
/// allocation suffers from internal fragmentation: wasted space whose
/// size is equal to the difference between the cell size and the object
/// size.
pub struct Block {}

impl Block {
    /// Get block header
    pub fn header(&self) -> &mut BlockHeader {
        unsafe { &mut *self.atoms().offset(END_ATOM as _).cast() }
    }
    /// Atom offset from pointer
    pub fn atom_number(&self, p: Address) -> u32 {
        let atom_n = self.candidate_atom_number(p);
        atom_n as _
    }
    /// Atom offset from pointer, might be wrong
    pub fn candidate_atom_number(&self, p: Address) -> usize {
        return (p.to_usize() - self as *const Self as usize) / ATOM_SIZE;
    }
    /// Pointer to atoms
    pub fn atoms(&self) -> *mut Atom {
        self as *const Self as *mut Atom
    }
    /// Is pointer aligned to atom size?
    pub const fn is_atom_aligned(p: Address) -> bool {
        (p.to_usize() & ATOM_ALIGNMENT_MASK) == 0
    }
    /// Try to get block from pointer
    pub fn from_cell(p: Address) -> *mut Self {
        (p.to_usize() & (!(BLOCK_SIZE - 1))) as *mut Self
    }
    /// Allocate new block and instantiate freelist.
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
            let mut count = 0;
            (&*memory).header().for_each_cell(|cell| {
                cell.to_mut_obj().zap(1);
                count += 1;
                (&mut *memory).header().freelist.free(cell);
            });
            (&*memory).header()
        }
    }
}

impl BlockHeader {
    /// Atoms per cell
    pub const fn atoms_per_cell(&self) -> usize {
        ((self.cell_size as usize + ATOM_SIZE - 1) / ATOM_SIZE) as _
    }
    /// Offset of end atom
    pub const fn end_atom(&self) -> usize {
        END_ATOM - self.atoms_per_cell() + 1
    }
    /// Cell size
    pub const fn cell_size(&self) -> u32 {
        self.cell_size
    }
    /// Start of the block
    pub fn begin(&self) -> Address {
        Address::from_ptr(self.block)
    }
    /// Iterate through each cell.
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
    /// Return cell at `index`
    pub fn cell(&self, index: usize) -> Address {
        self.begin().offset(index * self.cell_size() as usize)
    }
    /// Cell count
    pub const fn cell_count(&self) -> u32 {
        (BLOCK_SIZE as u32 - core::mem::size_of::<Self>() as u32) / self.cell_size()
    }
    /// Try to allocate memory of `cell_size` bytes.
    pub fn allocate(&mut self) -> Address {
        let addr = self.freelist.allocate();
        if addr.is_null() {
            self.can_allocate = false;
        }
        addr
    }
    /// Destroy this block
    pub fn destroy(&mut self) {
        unsafe {
            dealloc(
                self.block.cast(),
                Layout::from_size_align_unchecked(BLOCK_SIZE, BLOCK_SIZE),
            );
        }
    }
    /// Sweep this block.
    pub fn sweep(&mut self) -> bool {
        let mut is_empty = true;
        let mut freelist = FreeList::new();
        let mut count = 0;

        if GC_LOG {
            eprintln!(
                "--Sweep block {:p}, sie class: {}",
                self.block, self.cell_size
            );
        }
        let mut zcount = 0;
        let mut freed = 0;
        self.for_each_cell(|cell| {
            let object = cell.to_mut_obj();
            if !self.is_marked(cell) {
                count += 1;
                if !object.is_zapped() {
                    zcount += 1;
                    freed += object.trait_object().size() + core::mem::size_of::<Header>();
                    unsafe {
                        core::ptr::drop_in_place(object.trait_object());
                    }
                    object.zap(1);
                }
                freelist.free(cell);
            } else {
                is_empty = false;
                debug_assert!(self.is_marked(cell));
            }
        });
        if GC_LOG {
            eprintln!("--sweeped {} objects ({} bytes)", zcount, freed);
        }
        self.unswept = false;
        self.can_allocate = count != 0;

        self.freelist = freelist;
        is_empty
    }
    /// Test and set marked.
    pub fn test_and_set_marked(&mut self, p: Address) -> bool {
        /*self.bitmap
        .concurrent_test_and_set(self.atom_number(p) as _)*/
        let n = self.atom_number(p) as usize;
        if self.bitmap.get(n) {
            return true;
        }
        self.bitmap.set(n);
        false
    }
    /// Is pointer marked?
    pub fn is_marked(&self, p: Address) -> bool {
        self.bitmap.get(self.atom_number(p) as _)
    }
    /// Atom number
    pub fn atom_number(&self, p: Address) -> u32 {
        let atom_n = self.candidate_atom_number(p);
        atom_n as _
    }
    /// Atom number
    pub fn candidate_atom_number(&self, p: Address) -> usize {
        return (p.to_usize() - self.begin().to_usize()) / ATOM_SIZE;
    }
    /// Atoms pointer
    pub fn atoms(&self) -> *mut Atom {
        self.begin().to_mut_ptr()
    }
}
