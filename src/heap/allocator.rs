use crate::common::mem::Address;
use crate::common::ptr::DerefPointer;
use crate::common::space::*;
pub struct FreeListAllocator {
    pub space: Space,
    pub current_page: DerefPointer<Page>,
}

impl FreeListAllocator {
    pub fn new(heap: Space) -> Self {
        Self {
            space: heap,
            current_page: DerefPointer::null(),
        }
    }

    pub fn init(&mut self) {
        self.space.add_page(16 * 1024);
        self.current_page = DerefPointer::new(self.space.pages.first().unwrap());
    }

    pub fn allocate(&mut self, size: usize, must_alloc: bool) -> Address {
        let mut current_page = DerefPointer::null();
        let mut ptr = None;
        for page in self.space.pages.iter_mut() {
            if page.may_allocate(size) {
                current_page = DerefPointer::new(page);
                page.used = true;
                ptr = Some(page.bump(size));
            }
        }
        if ptr.is_some() {
            self.current_page = current_page;
        }
        ptr.unwrap_or_else(|| {
            /* slow path */
            for page in self.space.pages.iter_mut() {
                page.sweep();
                let (free_space, size) = page.freelist.alloc(size);
                if free_space.is_non_null() {
                    let object = free_space.addr();
                    let free_size = size;

                    let free_start = object.offset(size);
                    let free_end = object.offset(free_size);
                    let new_free_size = free_end.offset_from(free_start);
                    if new_free_size != 0 && new_free_size >= 16 {
                        page.freelist.add(free_start, new_free_size);
                    }
                    self.current_page.used = true;
                    self.current_page = DerefPointer::new(page);
                    return object;
                }
            }
            if must_alloc {
                self.space.add_page(16 * 1024);
                self.current_page = DerefPointer::new(self.space.pages.last().unwrap());
                self.current_page.used = true;
                return self.current_page.bump(size);
            }
            return Address::null();
        })
    }
}
