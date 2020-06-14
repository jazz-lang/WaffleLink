use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Block {
    pub size: usize,
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
    #[inline]
    pub unsafe fn from_pointer<'a, T>(ptr: *const T) -> Option<&'a mut Self> {
        if ptr.is_null() {
            return None;
        }
        let x = ptr as usize;
        let candidate = BLOCK_MASK ^ x;
        if candidate == 0 {
            return None;
        }
        Some(&mut *(candidate as *mut Self))
    }
}

pub struct BlockHeader {
    pub mark: AtomicBool,
    pub block: *mut Block,
}
