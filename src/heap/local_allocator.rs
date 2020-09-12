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
    current_block: Option<&'static mut MarkedBlockHandle>,
    last_active_block: Option<&'static mut MarkedBlockHandle>,
    /// After you do something to a block based on one of these cursors, you clear the bit in the
    /// corresponding bitvector and leave the cursor where it was.
    pub alloc_cursor: u32,
}

impl LocalAllocator {
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
            let res =
                dir.find_blocks_for_allocation(self, |this, block| this.try_allocate_in(block));
            res
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
