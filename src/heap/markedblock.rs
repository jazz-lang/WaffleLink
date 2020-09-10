use bit_set::*;

/// A marked block is a page-aligned container for heap-allocated objects.
/// Objects are allocated within cells of the marked block. For a given
/// marked block, all cells have the same size. Objects smaller than the
/// cell size may be allocated in the marked block, in which case the
/// allocation suffers from internal fragmentation: wasted space whose
/// size is equal to the difference between the cell size and the object
/// size.
pub struct MarkedBlock {}

pub const BLOCK_SIZE: usize = 16 * 1024;
pub const ATOM_SIZE: usize = 16;
pub const BLOCK_MASK: usize = !(BLOCK_SIZE - 1);
pub const ATOMS_PER_BLOCK: usize = BLOCK_SIZE / ATOM_SIZE;
pub const MAX_NUMBER_OF_LOWER_TIER_CELLS: usize = 8;
pub const END_ATOM: usize = (BLOCK_SIZE - core::mem::size_of::<Footer>()) / ATOM_SIZE;
pub const PAYLOAD_SIZE: usize = END_ATOM * ATOM_SIZE;
pub const FOOTER_SIZE: usize = BLOCK_SIZE - PAYLOAD_SIZE;
pub const ATOM_ALIGNMENT_MASK: usize = ATOM_SIZE - 1;
const_assert!(PAYLOAD_SIZE == (BLOCK_SIZE - core::mem::size_of::<Footer>()) & !(ATOM_SIZE - 1));

pub type Atom = [u8; ATOM_SIZE];

/// A per block object map.
struct ObjectMap {
    set: BitSet<u32>,
}

impl ObjectMap {
    /// Create a new `ObjectMap`.
    fn new() -> ObjectMap {
        ObjectMap {
            set: BitSet::with_capacity(BLOCK_SIZE / ATOM_SIZE),
        }
    }

    /// Set the address as a valid object.
    fn set_object(&mut self, atom_n: u32) {
        self.set.insert(atom_n as _);
    }

    /// Unset the address as a valid object.
    fn unset_object(&mut self, atom_n: u32) {
        self.set.remove(atom_n as _);
    }

    /// Return `true` is the address is a valid object.
    fn is_set(&self, atom_n: u32) -> bool {
        self.set.contains(atom_n as _)
    }

    /// Update this `ObjectMap` with the difference of this `ObjectMap` and
    /// the other.
    fn difference(&mut self, other: &ObjectMap) {
        self.set.difference_with(&other.set);
    }

    /// Clear all entries.
    fn clear(&mut self) {
        self.set.clear();
    }
}

#[repr(C)]
pub struct MarkedBlockHandle {
    atoms_per_cell: u32,
    end_atom: u32,
    is_freelisted: bool,
    index: u32,
    block: *mut MarkedBlock,
    pub can_allocate: bool,
    pub empty: bool,
}

pub struct Footer {
    MarkedBlockHandle: &'static mut MarkedBlockHandle,
    marking_version: u32,
    newly_allocated_version: u32,
    marks: ObjectMap,
    newly_allocated: ObjectMap,
}

impl MarkedBlock {
    pub fn atoms(&self) -> *mut Atom {
        self as *const Self as *mut _
    }

    pub fn footer(&self) -> &'static mut Footer {
        unsafe { &mut *self.atoms().offset(END_ATOM as _).cast() }
    }

    pub fn handle(&self) -> &'static mut MarkedBlockHandle {
        self.footer().MarkedBlockHandle
    }

    pub fn is_atom_aligned(p: *const ()) -> bool {
        (p as usize & ATOM_ALIGNMENT_MASK) == 0
    }
    pub fn candidate_atom_number(&self, p: *const ()) -> usize {
        return (p as usize - self as *const Self as usize) / ATOM_SIZE;
    }

    pub fn atom_number(&self, p: *const ()) -> u32 {
        let atom_n = self.candidate_atom_number(p);
        assert!(atom_n < self.handle().end_atom as usize);
        atom_n as _
    }
    pub fn is_markedv(&self, version: u32, p: *const ()) -> bool {
        let v = self.footer().marking_version;
        if version != v {
            return false;
        }
        crate::utils::atomics::load_load_fence();
        self.footer().marks.is_set(self.atom_number(p))
    }

    pub fn is_marked(&self, p: *const ()) -> bool {
        self.footer().marks.is_set(self.atom_number(p))
    }
}

impl MarkedBlockHandle {
    pub fn cell_align(&self, p: *const ()) -> *const () {
        let base = self.block().atoms() as usize;
        let mut bits = p as usize;
        bits -= base;
        bits -= bits % self.cell_size();
        bits += base;
        bits as *const ()
    }

    pub fn cell_size(&self) -> usize {
        self.atoms_per_cell as usize * ATOM_SIZE
    }
    pub fn block(&self) -> &MarkedBlock {
        unsafe { &*self.block }
    }
    pub fn block_footer(&self) -> &mut Footer {
        self.block().footer()
    }
}
