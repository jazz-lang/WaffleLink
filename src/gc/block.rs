use super::*;
use freelist::*;
use std::sync::atomic::{AtomicBool, Ordering};
pub struct Block {
    pub size: usize,
    pub freelist: FreeList,
    pub memory: Address,
    pub start: Address,
    pub cursor: Address,
    pub limit: Address,
}

pub const BLOCK_SIZE: usize = 32 * 1024;
pub const BLOCK_MASK: usize = !(BLOCK_SIZE - 1);
use std::alloc::*;
impl Block {
    pub fn boxed() -> Box<Self> {
        let layout = Layout::from_size_align(BLOCK_SIZE, BLOCK_SIZE).unwrap();
        let mem = unsafe { alloc_zeroed(layout).cast::<BlockHeader>() };
        let mut block = Box::new(Block {
            size: BLOCK_SIZE,
            freelist: FreeList::new(),
            memory: Address::from_ptr(mem),
            start: Address::from_ptr(mem).offset(std::mem::size_of::<BlockHeader>()),
            cursor: Address::from_ptr(mem).offset(std::mem::size_of::<BlockHeader>()),
            limit: Address::from_ptr(mem).offset(BLOCK_SIZE),
        });
        unsafe {
            mem.write(BlockHeader {
                mark: AtomicBool::new(false),
                block: &mut *block,
            })
        }

        block
    }
    pub unsafe fn header(&self) -> &mut BlockHeader {
        &mut *self.memory.to_mut_ptr::<BlockHeader>()
    }
    pub fn is_marked(&self) -> bool {
        unsafe { self.header().mark.load(Ordering::Acquire) }
    }
    /// Invoke finalizers and build free-list.
    pub unsafe fn sweep(&self) -> bool {
        let start = self.start;
        let end = self.limit;
        let mut scan = start;
        let mut garbage_start = Address::null();
        let mut freelist = FreeList::new();
        let mut add_freelist = |start: Address, end: Address| {
            if start.is_null() {
                return;
            }
            let size = end.offset_from(start);
            freelist.add(start, size);
        };
        let mut all_free = true;
        while scan < self.cursor {
            let object = WaffleCellPointer::from_ptr(scan.to_mut_ptr::<WaffleCell>());
            if object.value().header().is_marked() {
                add_freelist(garbage_start, scan);
                object.value_mut().header_mut().unmark();
                assert!(!object.value().header().is_marked());
                all_free = false;
                log::trace!(
                    "marked ptr {:p} it is {:?}",
                    scan.to_mut_ptr::<u8>(),
                    object.value().header().ty
                );
            } else if garbage_start.is_non_null() {
                assert!(!object.value().header().is_marked());
                //log::trace!("unmarked ptr {:p}", scan.to_mut_ptr::<u8>());
                object.finalize();
            } else {
                assert!(!object.value().header().is_marked());
                log::trace!("unmarked ptr {:p}", scan.to_mut_ptr::<u8>());
                object.finalize();
                garbage_start = scan;
            }
            if object.value_mut().header_mut().fwdptr() == 0 {
                scan = scan.offset(1);
                continue;
            }
            //println!("{}", object.value_mut().header_mut().fwdptr());
            scan = scan.offset(object.value_mut().header_mut().fwdptr());
        }

        add_freelist(garbage_start, end);
        self.header().mark.store(false, Ordering::Relaxed);
        all_free
    }
    #[inline]
    pub unsafe fn try_from_pointer<'a, T>(ptr: *const T) -> Option<*mut Self> {
        if ptr.is_null() {
            return None;
        }
        let x = ptr as usize;
        let candidate = x & BLOCK_MASK;
        if candidate == 0 {
            return None;
        }
        let b = candidate as *mut BlockHeader;

        Some((&*b).block)
    }
    #[inline]
    pub unsafe fn from_pointer<'a, T>(ptr: *const T) -> Option<&'a mut Self> {
        if ptr.is_null() {
            return None;
        }
        let x = ptr as usize;
        let candidate = x & BLOCK_MASK;
        if candidate == 0 {
            return None;
        }
        let b = candidate as *mut BlockHeader;

        Some(&mut *((&*b).block))
    }
    pub unsafe fn allocate(&mut self, size: usize) -> Address {
        let addr = self.freelist.alloc(size);
        if addr.is_non_null() {
            let ptr = addr.addr();
            let hdr = (&mut *ptr.to_mut_ptr::<WaffleCell>()).header_mut();
            hdr.fwdptr = addr.size();
            hdr.unmark();

            return ptr;
        }
        if self.cursor.offset(size) < self.limit {
            let ptr = self.cursor;
            self.cursor = self.cursor.offset(size);
            let hdr = (&mut *self.cursor.to_mut_ptr::<WaffleCell>()).header_mut();
            //println!("{}", self.limit.to_usize() - self.cursor.to_usize());
            hdr.fwdptr = self.limit.to_usize() - self.cursor.to_usize();
            hdr.unmark();
            hdr.ty = WaffleType::None;
            return ptr;
        }
        return Address::null();
    }
}

pub struct BlockHeader {
    pub mark: AtomicBool,
    pub block: *mut Block,
}
