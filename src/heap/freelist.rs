use super::heap_cell::*;
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct FreeCell {
    pub preserved_bits: u64,
    pub scrambled_next: usize,
}

impl FreeCell {
    pub fn scramble(cell: *const Self, secret: usize) -> usize {
        cell as usize ^ secret
    }
    pub fn descramble(cell: usize, secret: usize) -> *mut Self {
        (cell ^ secret) as *mut _
    }

    pub fn set_next(&mut self, next: *const Self, secret: usize) {
        self.scrambled_next = Self::scramble(next, secret);
    }

    pub fn next(&self, secret: usize) -> *mut Self {
        Self::descramble(self.scrambled_next, secret)
    }

    pub fn offset_of_scrambled_next() -> usize {
        offset_of!(Self, scrambled_next)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct FreeList {
    scramble_head: usize,
    secret: usize,
    payload_end: *mut u8,
    remaining: usize,
    original_size: usize,
    cell_size: usize,
}
impl FreeList {
    pub fn cell_size(&self) -> usize {
        self.cell_size
    }

    offset_of_field_fn!(cell_size);
    offset_of_field_fn!(original_size);
    offset_of_field_fn!(remaining);
    offset_of_field_fn!(payload_end);
    offset_of_field_fn!(secret);
    offset_of_field_fn!(scramble_head);
    pub fn allocation_will_fail(&self) -> bool {
        return self.head().is_null() && self.remaining == 0;
    }

    pub fn allocation_will_succeed(&self) -> bool {
        return !self.allocation_will_fail();
    }
    pub(super) fn head(&self) -> *mut FreeCell {
        FreeCell::descramble(self.scramble_head, self.secret)
    }
    #[inline(always)]
    pub fn allocate(&mut self, mut slow_path: impl FnMut() -> *mut HeapCell) -> *mut HeapCell {
        let mut remaining = self.remaining;
        if remaining > 0 {
            let cell_size = self.cell_size;
            remaining = remaining.wrapping_sub(cell_size);
            self.remaining = remaining;
            return (self.payload_end as isize - remaining as isize - cell_size as isize) as *mut _;
        }
        let result = self.head();
        if unlikely!(result.is_null()) {
            return slow_path();
        }

        self.scramble_head = unsafe { (*result).scrambled_next };
        result as *mut _
    }

    pub fn for_each(&self, mut f: impl FnMut(*mut HeapCell)) {
        if self.remaining > 0 {
            let mut remaining = self.remaining;
            while remaining > 0 {
                f((self.payload_end as isize - remaining as isize) as *mut _);
                remaining = remaining.wrapping_sub(self.cell_size);
            }
        } else {
            unsafe {
                let mut cell = self.head();
                while !cell.is_null() {
                    let next = (*cell).next(self.secret);
                    f(cell as *mut _);
                    cell = next;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.scramble_head = 0;
        self.secret = 0;
        self.payload_end = std::ptr::null_mut();
        self.remaining = 0;
        self.original_size = 0;
    }

    pub fn initialize_list(&mut self, head: *mut FreeCell, secret: usize, bytes: usize) {
        self.scramble_head = FreeCell::scramble(head, secret);
        self.secret = secret;
        self.payload_end = std::ptr::null_mut();
        self.remaining = 0;
        self.original_size = bytes;
    }

    pub fn initialize_bump(&mut self, end: *mut u8, remaining: usize) {
        self.scramble_head = 0;
        self.secret = 0;
        self.payload_end = end;
        self.remaining = remaining;
        self.original_size = remaining;
    }

    pub fn contains(&self, target: *mut HeapCell) -> bool {
        if self.remaining > 0 {
            let start = self.payload_end as isize - self.remaining as isize;
            let end = self.payload_end as isize;
            return start <= target as isize && (target as isize) < end;
        }
        let mut candidate = self.head();
        while !candidate.is_null() {
            if candidate as usize == target as usize {
                return true;
            }
            candidate = unsafe { (*candidate).next(self.secret) };
        }
        false
    }

    pub const fn new(size: usize) -> Self {
        Self {
            cell_size: size,
            secret: 0,
            payload_end: std::ptr::null_mut(),
            original_size: 0,
            remaining: 0,
            scramble_head: 0,
        }
    }
}
