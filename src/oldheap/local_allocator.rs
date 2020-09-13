use super::*;
use block_directory::*;
use freelist::*;
use intrusive_collections::{intrusive_adapter, LinkedListLink, UnsafeRef};
use markedblock::*;
intrusive_adapter!(pub LAdapter = UnsafeRef<LocalAllocator> : LocalAllocator {link :LinkedListLink});
pub struct LocalAllocator {
    link: LinkedListLink,
    directory: *mut BlockDirectory,
    freelist: FreeList,
    current_block: Option<*mut MarkedBlockHandle>,
    last_active_block: Option<*mut MarkedBlockHandle>,
    /// After you do something to a block based on one of these cursors, you clear the bit in the
    /// corresponding bitvector and leave the cursor where it was.
    pub alloc_cursor: u32,
}

impl LocalAllocator {
    pub fn reset(&mut self) {
        self.alloc_cursor = 0;
        self.freelist.clear();
        self.current_block = None;
        self.last_active_block = None;
    }

    pub fn new(directory: *mut BlockDirectory) -> *mut Self {
        unsafe {
            let dir = &mut *directory;
            let mut this = Self {
                directory,
                freelist: FreeList::new(dir.cell_size() as _),
                last_active_block: None,
                current_block: None,
                link: LinkedListLink::new(),
                alloc_cursor: 0,
            };

            let lock = dir.local_allocators_lock.lock();
            dir.local_allocators.push_back(this);
            dir.local_allocators.back_mut().unwrap() as *mut Self
        }
    }

    pub fn stop_allocating(&mut self) {
        if self.current_block.is_none() {
            return;
        }
        unsafe {
            (&mut *self.current_block.unwrap()).stop_allocating(&mut self.freelist);
        }
        self.last_active_block = self.current_block;
        self.current_block = None;
        self.freelist.clear();
    }

    pub fn resume_allocating(&mut self) {
        if self.last_active_block.is_none() {
            return;
        }

        unsafe {
            (&mut *self.last_active_block.unwrap()).resume_allocating(&mut self.freelist);
        }
        self.current_block = self.last_active_block;
        self.last_active_block = None;
    }

    pub fn prepare_for_allocation(&mut self) {
        self.reset();
    }

    pub fn stop_allocating_for_good(&mut self) {
        self.stop_allocating();
        self.reset();
    }

    pub fn directory(&self) -> &mut BlockDirectory {
        unsafe { &mut *self.directory }
    }
    #[inline]
    pub fn allocate(&mut self) -> *mut () {
        unsafe {
            // This is dumb: rustc is not smart enough to find out that `self.allocate_slow_case()` is invoked when we actually
            // do not borrow freelist anymore.
            let mut this = &mut *(self as *mut Self);
            this.freelist.allocate(|| self.allocate_slow_case())
        }
    }

    fn try_to_allocate_without_gc(&mut self) -> *mut () {
        unsafe {
            let dir = &mut *self.directory;
            loop {
                let block = dir.find_block_for_allocation(self);
                if block.is_null() {
                    return 0 as *mut ();
                }
                let res = self.try_allocate_in(block);
                if !res.is_null() {
                    return res;
                }
            }
        }
    }
    fn allocate_slow_case(&mut self) -> *mut () {
        let result = self.try_to_allocate_without_gc();
        if !result.is_null() {
            return result;
        }

        let block = self.directory().create_block();
        self.directory().add_block(block);
        let res = self.try_allocate_in(block);
        assert!(!res.is_null());
        res
    }
    fn try_allocate_in(&mut self, p: *mut MarkedBlockHandle) -> *mut () {
        unsafe {
            let handle = &mut *p;

            handle.sweep(&mut self.freelist, handle.empty);
            if self.freelist.allocation_will_fail() {
                return 0 as *mut ();
            }
            let result = self.freelist.allocate(|| {
                unreachable!();
            });
            result
        }
    }
}
