use super::mem::*;
use crate::gc::*;
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
        //let even_bytes = bytes + (bytes & 0x01);
        let even_bytes = bytes;
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
        assert!(space.pages.is_empty() == false);
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
        }
    }

    pub fn uncommit(&self) {
        uncommit(self.data, self.size)
    }
}
