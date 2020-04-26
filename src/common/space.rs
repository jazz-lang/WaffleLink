use super::freelist::FreeList;
use super::mem::*;
use crate::common::ptr::*;
use crate::runtime::cell::*;
use crate::runtime::process::local_data;
pub struct Space {
    pub top: Address,
    pub limit: Address,
    pub pages: Vec<Page>,
    pub size: usize,
    pub size_limit: usize,
    pub page_size: usize,
    pub pages_count: usize,
    pub allocated_size: usize,
}

impl Space {
    pub fn empty() -> Self {
        Self {
            top: Address::null(),
            limit: Address::null(),
            pages: Vec::new(),
            size: 0,
            allocated_size: 0,
            page_size: 0,
            pages_count: 0,
            size_limit: 0,
        }
    }
    pub fn new(page_size: usize) -> Self {
        let mut pages = Vec::new();
        let page = Page::new(page_size);
        pages.push(page);
        let top = Address::from_ptr(&pages.last().unwrap().top);
        let limit = Address::from_ptr(&pages.last().unwrap().limit);
        let mut space = Space {
            top,
            limit,
            pages,
            size: 0,
            page_size,
            size_limit: 0,
            pages_count: 1,
            allocated_size: 0,
        };
        space.compute_size_limit();
        space
    }

    pub fn compute_size_limit(&mut self) {
        self.size_limit = self.size << 1;
    }
    pub fn may_allocate_in_current(&mut self, size: usize) -> bool {
        let even_bytes = size + (size & 0x01);
        let place_in_current = self.top.deref().offset(even_bytes) <= self.limit.deref();
        place_in_current
    }
    pub fn add_page(&mut self, size: usize) {
        let real_size = align_usize(size, page_size());
        let page = Page::new(real_size);
        self.pages.push(page);
        self.pages_count += 1;
        let page = self.pages.last().unwrap();
        self.size += real_size;
        self.top = Address::from_ptr(&page.top);
        self.limit = Address::from_ptr(&page.limit);
    }

    pub fn fast_allocate(&mut self, bytes: usize, needs_gc: &mut bool) -> Address {
        let even_bytes = bytes + (bytes & 0x01);
        let place_in_current = self.top.deref().offset(even_bytes) < self.limit.deref();

        if !place_in_current {
            *needs_gc = true;
            log::debug!("Add new page");
            self.add_page(even_bytes);
        }
        self.allocated_size += even_bytes;
        let result = self.top.deref();
        unsafe {
            *self.top.to_mut_ptr::<*mut u8>() =
                self.top.deref().offset(even_bytes).to_mut_ptr::<u8>();
        }
        result
    }
    pub fn try_find_page_for(&self, size: usize) -> Option<(Address, Address)> {
        for page in self.pages.iter() {
            if page.top.offset(size) < page.limit {
                return Some((Address::from_ptr(&page.top), Address::from_ptr(&page.limit)));
            }
        }

        return None;
    }
    pub fn allocate(&mut self, bytes: usize, needs_gc: &mut bool) -> Address {
        let even_bytes = bytes + (bytes & 0x01);
        let place_in_current = self.top.deref().offset(even_bytes) < self.limit.deref();

        if !place_in_current {
            let head = self.try_find_page_for(even_bytes);

            if let None = head {
                *needs_gc = true;
                self.add_page(even_bytes);
            } else if let Some((top, limit)) = head {
                self.top = top;
                self.limit = limit;
            }
        }
        self.allocated_size += even_bytes;
        let result = self.top.deref();
        unsafe {
            *self.top.to_mut_ptr::<*mut u8>() =
                self.top.deref().offset(even_bytes).to_mut_ptr::<u8>();
        }
        result
    }

    pub fn swap(&mut self, space: &mut Space) {
        self.clear();
        while space.pages.is_empty() != true {
            self.pages.push(space.pages.pop().unwrap());
            self.size += self.pages.last().unwrap().size;
        }
        self.allocated_size = space.allocated_size;
        let page = self.pages.last().unwrap();
        self.top = Address::from_ptr(&page.top);
        self.limit = Address::from_ptr(&page.limit);
    }

    pub fn contains(&self, addr: Address) -> bool {
        for page in self.pages.iter() {
            let page: &Page = page;
            if addr >= page.data && addr <= page.limit {
                return true;
            }
        }

        false
    }
    pub fn clear(&mut self) {
        self.size = 0;
        while let Some(page) = self.pages.pop() {
            page.uncommit();
        }
    }
}

pub struct Page {
    pub data: Address,
    pub top: Address,
    pub limit: Address,
    pub size: usize,
    pub needs_sweep: bool,
    pub freelist: FreeList,
    pub used: bool,
}

impl Page {
    pub fn new(size: usize) -> Self {
        let data = commit(size, false);
        let top = data;
        let limit = data.offset(size);
        Self {
            top,
            data,
            limit,
            size,
            needs_sweep: false,
            freelist: FreeList::new(),
            used: false,
        }
    }
    fn add_freelist(&mut self, start: Address, size: usize) {
        self.freelist.add(start, size);
    }
    pub fn sweep(&mut self) -> bool {
        if !self.needs_sweep {
            return false;
        }
        let mut all_free = true;
        let page = self;
        let start = page.data;
        let end = page.top;
        macro_rules! add_freelist {
            ($start: expr,$end: expr) => {
                if !$start.is_null() {
                    let size = $end.offset_from($start);
                    page.add_freelist($start, size);
                    true
                } else {
                    false
                }
            };
        }
        let mut scan = start;
        let mut garbage_start = Address::null();
        while scan < end {
            let mut cell = Ptr::<Cell>::from_raw(page.data.to_mut_ptr::<Cell>());
            if cell.color == CELL_BLACK {
                all_free = false;
                log::trace!("Re-white '{:p}'", cell.raw);
                add_freelist!(garbage_start, scan);
                garbage_start = Address::null();
                cell.color = CELL_WHITE_A;
            } else if garbage_start.is_non_null() {
                log::trace!("Sweep '{:p}'", cell.raw);
                unsafe {
                    std::ptr::drop_in_place(cell.raw);
                }
            } else {
                log::trace!("Sweep '{:p}'", cell.raw);
                garbage_start = scan;
                unsafe {
                    std::ptr::drop_in_place(cell.raw);
                }
            }
            scan = scan.offset(std::mem::size_of::<Cell>());
        }
        add_freelist!(garbage_start, end);
        page.needs_sweep = false;
        if all_free {
            page.used = false;
            page.top = page.data;
        }
        all_free
    }

    pub fn bump(&mut self, size: usize) -> Address {
        let ptr = self.top;
        self.top = self.top.offset(size);
        ptr
    }

    pub fn may_allocate(&self, size: usize) -> bool {
        self.top.offset(size) <= self.limit
    }
    pub fn uncommit(&self) {
        uncommit(self.data, self.size)
    }
}
