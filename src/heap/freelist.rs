//! # Bump'n'pop
//!
//! Hybrid bump-pointer/free-list allocator.

#[repr(C)]
pub struct FreeCell {
    preserved_bits: u64,
    next: *mut Self,
}

impl FreeCell {
    pub fn set_next(&mut self, cell: *mut Self) {
        self.next = cell;
    }

    pub fn next(&self) -> *mut Self {
        self.next
    }
}

pub struct FreeList {
    head: *mut FreeCell,
    payload_end: usize,
    remaining: u32,
    original_size: u32,
    cell_size: u32,
}
impl FreeList {
    pub const fn new(cell_size: u32) -> Self {
        Self {
            cell_size,
            head: core::ptr::null_mut(),
            remaining: 0,
            payload_end: 0,
            original_size: 0,
        }
    }

    pub fn clear(&mut self) {
        self.head = 0 as *mut _;
        self.payload_end = 0;
        self.remaining = 0;
        self.original_size = 0;
    }

    pub fn initialize_list(&mut self, head: *mut FreeCell, bytes: u32) {
        self.head = head;
        self.payload_end = 0;
        self.remaining = 0;
        self.original_size = bytes;
    }

    pub fn initialize_bump(&mut self, end: usize, remaining: u32) {
        self.head = 0 as *mut _;
        self.payload_end = end;
        self.remaining = remaining;
        self.original_size = remaining;
    }
    pub fn contains(&self, p: *const ()) -> bool {
        if self.remaining != 0 {
            let start = (self.payload_end as isize - self.remaining as isize) as usize;
            let end = self.payload_end;
            return (start <= p as usize) && ((p as usize) < end);
        }
        let mut candidate = self.head;
        while !candidate.is_null() {
            if candidate as *const () == p {
                return true;
            }
            candidate = unsafe { (&*candidate).next() };
        }

        false
    }
    pub fn allocation_will_fail(&self) -> bool {
        self.head.is_null() || self.remaining == 0
    }

    pub fn allocate(&mut self, mut slow_path: impl FnMut() -> *mut ()) -> *mut () {
        let mut remaining = self.remaining;
        if remaining != 0 {
            let cell_size = self.cell_size();
            remaining -= cell_size;
            self.remaining = remaining;
            return (self.payload_end as isize - remaining as isize - cell_size as isize)
                as *mut ();
        }
        let result = self.head;
        #[cold]
        if result.is_null() {
            return slow_path();
        }
        self.head = unsafe { (&*result).next };
        result.cast()
    }
    pub fn cell_size(&self) -> u32 {
        self.cell_size
    }

    pub fn original_size(&self) -> u32 {
        self.original_size
    }
}
