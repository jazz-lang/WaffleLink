use super::bitmap::BitMap;
use super::block_directory::*;
use super::freelist::*;
use super::object::*;
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

#[repr(C)]
pub struct MarkedBlockHandle {
    pub atoms_per_cell: u32,
    pub end_atom: u32,
    pub is_freelisted: bool,
    pub directory: *mut BlockDirectory,
    pub index: u32,
    pub block: *mut MarkedBlock,
    pub can_allocate: bool,
    pub empty: bool,
}

pub struct Footer {
    handle: &'static mut MarkedBlockHandle,
    marking_version: u32,
    newly_allocated_version: u32,
    marks: BitMap,
    newly_allocated: BitMap,
}

impl MarkedBlock {
    pub fn create() -> &'static mut MarkedBlockHandle {
        unsafe {
            let block_space = std::alloc::alloc(
                std::alloc::Layout::from_size_align(BLOCK_SIZE, BLOCK_SIZE).unwrap(),
            );
            let mut handle = Box::new(MarkedBlockHandle {
                block: block_space.cast(),
                atoms_per_cell: 0,
                end_atom: 0,
                empty: true,
                directory: 0 as *mut _,
                can_allocate: true,
                index: 0,
                is_freelisted: false,
            });
            let raw = Box::into_raw(handle);
            let mut handle: &'static mut MarkedBlockHandle = &mut *raw;
            let block = handle.block;
            *(&mut *block).footer() = Footer {
                handle,
                marking_version: 0,
                marks: BitMap::new(),
                newly_allocated: BitMap::new(),
                newly_allocated_version: 0,
            };
            &mut *raw
        }
    }

    pub fn atoms(&self) -> *mut Atom {
        self as *const Self as *mut _
    }

    pub fn footer(&self) -> &'static mut Footer {
        unsafe { &mut *self.atoms().offset(END_ATOM as _).cast() }
    }

    pub fn handle(&self) -> &'static mut MarkedBlockHandle {
        self.footer().handle
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
        self.footer().marks.get(self.atom_number(p) as _)
    }

    pub fn is_marked(&self, p: *const ()) -> bool {
        self.footer().marks.get(self.atom_number(p) as _)
    }

    pub fn test_and_set_marked(&self, p: *const ()) -> bool {
        self.footer()
            .marks
            .concurrent_test_and_set(self.atom_number(p) as _)
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
    pub fn did_add_to_directory(&mut self, directory: *mut BlockDirectory, index: u32) {
        unsafe {
            let dir = &mut *directory;
            self.index = index;
            self.directory = directory;
            let cell_size = dir.cell_size();
            self.atoms_per_cell = ((cell_size + ATOM_SIZE - 1) / ATOM_SIZE) as _;
            self.end_atom = (END_ATOM - self.atoms_per_cell as usize + 1) as _;
        }
    }
    pub fn did_remove_from_directory(&mut self) {
        self.index = u32::MAX;
        self.directory = 0 as *mut _;
    }

    pub fn sweep(&mut self, freelist: &mut FreeList, empty: bool) {
        unsafe {
            let start_of_last_cell = self.cell_align(
                self.block()
                    .atoms()
                    .offset(self.end_atom as isize - 1)
                    .cast(),
            );
            let payload_end = start_of_last_cell.offset(self.cell_size() as _);
            let payload_begin = self.block().atoms().cast::<()>();
            if empty {
                self.is_freelisted = true;
                freelist.initialize_bump(
                    payload_end as _,
                    (payload_end as isize - payload_begin as isize) as u32,
                );
                if super::GC_LOG {
                    eprintln!(
                        "--Quickly swept block {:p} with cell size {}",
                        self.block,
                        self.cell_size()
                    );
                }
            }
            // If cell is not zapped invokes `drop` on it and zaps it, otherwise it is just no-op.
            let mut destroy = |addr: *mut ()| {
                let gcbox = addr.cast::<GcBox<()>>();
                let gc = &mut *gcbox;
                if !gc.is_zapped() {
                    std::ptr::drop_in_place(gc.trait_object());
                    gc.zap(1);
                }
            };

            let mut count = 0;
            let mut head: *mut FreeCell = 0 as *mut _;
            let mut handle_dead_cell = |ix: usize| {
                let cell = self.block().atoms().offset(ix as _).cast::<()>();

                destroy(cell);
                let free_cell = cell.cast::<FreeCell>();
                (&mut *free_cell).set_next(head);
                head = free_cell;
                count += 1;
            };
            let mut is_empty = true;
            let mut i = 0;
            while i < self.end_atom {
                if self.block_footer().marks.get(i as _) {
                    is_empty = false;
                    continue;
                }
                handle_dead_cell(i as _);
                i += self.atoms_per_cell;
            }

            if is_empty {
                self.empty = true;
            } else {
                freelist.initialize_list(head, count * self.cell_size() as u32);
                self.is_freelisted = true;
            }

            if super::GC_LOG {
                eprintln!(
                    "Slowly swept block {:p} with cell size {}",
                    self.block,
                    self.cell_size()
                );
            }
        }
    }
}

impl Drop for MarkedBlockHandle {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(
                self.block.cast(),
                std::alloc::Layout::from_size_align(BLOCK_SIZE, BLOCK_SIZE).unwrap(),
            );
        }
    }
}
