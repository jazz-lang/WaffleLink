pub mod allocator;
pub mod block_allocator;
pub mod block_info;
use crate::gc::{constants::*, *};
use allocator::local_allocator::*;
use allocator::*;
use block_allocator::BlockAllocator;
use block_info::BlockInfo;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
pub struct ImmixSpace {
    id: AtomicUsize,
    pub(super) block_allocator: Arc<BlockAllocator>,
    pub(super) allocators: Arc<Mutex<Vec<Arc<UnsafeCell<LocalAllocator>>>>>,
    /// The current live mark for new objects. See `Heap.current_live_mark`.
    current_live_mark: AtomicBool,
    pub(super) evac_allocator: Arc<Mutex<evac_allocator::EvacAllocator>>,
}

impl ImmixSpace {
    pub fn local_allocator(&self) -> &mut LocalAllocator {
        unsafe { LOCAL_ALLOCATOR.with(|x| &mut *x.get()) }
    }

    pub fn new() -> Self {
        Self {
            id: AtomicUsize::new(0),
            block_allocator: Arc::new(BlockAllocator::new()),
            allocators: Arc::new(Mutex::new(vec![])),
            current_live_mark: AtomicBool::new(false),
            evac_allocator: Arc::new(Mutex::new(evac_allocator::EvacAllocator::new())),
        }
    }
    /// Get the block for the given object.
    unsafe fn get_block_ptr(object: GCObjectRef) -> *mut BlockInfo {
        let block_offset = object.raw() as usize % BLOCK_SIZE;
        let block = std::mem::transmute((object.raw() as *mut u8).offset(-(block_offset as isize)));
        /*log::debug!(
            "Block for object {:p}: {:p} with offset: {}",
            object.raw(),
            block,
            block_offset
        );*/
        block
    }
    /// Decrement the lines on which the object is allocated.
    pub fn decrement_lines(&self, object: GCObjectRef) {
        log::debug!("decrement_lines() on object {:p}", object.raw());
        debug_assert!(
            self.is_gc_object(object),
            "decrement_lines() on invalid object {:p}",
            object.raw()
        );
        unsafe {
            (*ImmixSpace::get_block_ptr(object)).decrement_lines(object);
        }
    }

    /// Increment the lines on which the object is allocated.
    pub fn increment_lines(&self, object: GCObjectRef) {
        log::debug!("increment_lines() on object {:p}", object.raw());
        debug_assert!(
            self.is_gc_object(object),
            "increment_lines() on invalid object {:p}",
            object.raw()
        );
        unsafe {
            (*ImmixSpace::get_block_ptr(object)).increment_lines(object);
        }
    }
    pub fn evac_headroom(&self) -> usize {
        self.evac_allocator.lock().evac_headroom()
    }
    /// Set an address in this space as a valid object.
    pub fn set_gc_object(&self, object: GCObjectRef) {
        log::debug!("set_gc_object() on object {:p}", object.raw());
        debug_assert!(
            self.block_allocator.is_in_space(object),
            "set_gc_object() on invalid object {:p}",
            object.raw()
        );
        unsafe {
            (*ImmixSpace::get_block_ptr(object)).set_gc_object(object);
        }
    }

    /// Unset an address as a valid object within the immix space.
    pub fn unset_gc_object(&self, object: GCObjectRef) {
        log::debug!("unset_gc_object() on object {:p}", object.raw());
        debug_assert!(
            self.block_allocator.is_in_space(object),
            "unset_gc_object() on invalid object {:p}",
            object.raw()
        );
        unsafe {
            (*ImmixSpace::get_block_ptr(object)).unset_gc_object(object);
        }
    }

    /// Return if the object an the address is a valid object within the immix
    /// space.
    pub fn is_gc_object(&self, object: GCObjectRef) -> bool {
        if self.block_allocator.is_in_space(object) {
            unsafe { (*ImmixSpace::get_block_ptr(object)).is_gc_object(object) }
        } else {
            false
        }
    }

    /// Return a closure that behaves like `ImmixSpace::is_gc_object()`.
    pub fn is_gc_object_filter<'a>(&'a self) -> Box<dyn Fn(GCObjectRef) -> bool + 'a> {
        let block_allocator = &self.block_allocator;
        Box::new(move |object: GCObjectRef| {
            block_allocator.is_in_space(object)
                && unsafe { (*ImmixSpace::get_block_ptr(object)).is_gc_object(object) }
        })
    }

    /// Return if the object an the address is within the immix space.
    pub fn is_in_space(&self, object: GCObjectRef) -> bool {
        self.block_allocator.is_in_space(object)
    }

    /// Return the number of unallocated blocks.
    pub fn available_blocks(&self) -> usize {
        self.block_allocator.available_blocks()
    }

    /// Return a collection of blocks to the global block allocator.
    pub fn return_blocks(&self, blocks: Vec<*mut BlockInfo>) {
        self.block_allocator.return_blocks(blocks);
    }

    /// Set the current live mark to `current_live_mark`.
    pub fn set_current_live_mark(&self, current_live_mark: bool) {
        self.current_live_mark
            .store(current_live_mark, Ordering::Release);
    }

    /// Set the recyclable blocks for the `NormalAllocator`.
    pub fn set_recyclable_blocks(&self, blocks: Vec<*mut BlockInfo>) {
        self.block_allocator.recycle(blocks);
    }

    /// Get all block managed by all allocators, draining any local
    /// collections.
    pub fn get_all_blocks(&self) -> Vec<*mut BlockInfo> {
        let mut blocks = vec![];
        for allocator in self.allocators.lock().iter() {
            unsafe {
                blocks.extend((&mut *allocator.get()).get_all_blocks());
            }
        }
        let ba = &self.block_allocator;
        //let mut recyc = ba.recyclable_blocks.clone();
        //ba.recyclable_blocks.clear();
        let mut evac_blocks = self.evac_allocator.lock().get_all_blocks();
        /*blocks
        .drain(..)
        .chain(ba.unavailable_blocks.drain(..))
        .chain(evac_blocks.drain(..))
        .chain(recyc.drain(..))
        .collect()*/
        /*while let Some(block) = ba.recyclable_blocks.pop() {
            blocks.push(block);
        }
        while let Some(block) = ba.unavailable_blocks.pop() {
            blocks.push(block);
        }*/
        blocks.extend(ba.drain_blocks());
        blocks.drain(..).chain(evac_blocks.drain(..)).collect()
    }

    /// Allocate an object of `size` bytes or return `None` if the allocation
    /// failed.
    ///
    /// This object is initialized and ready to use.
    pub fn allocate(&self, ty: WaffleType, size: usize) -> Option<GCObjectRef> {
        log::debug!("Request to allocate an object of size {}", size);

        if let Some(object) = self.local_allocator().allocate(size) {
            object.value_mut().header_mut().set_type(ty);
            object
                .value_mut()
                .header_mut()
                .mark(self.current_live_mark.load(Ordering::Relaxed));
            unsafe {
                (*ImmixSpace::get_block_ptr(object)).set_new_object(object);
            }
            object.value_mut().header_mut().set_new();
            self.set_gc_object(object);
            unsafe {
                super::object_init(ty, Address::from_ptr(object.raw()));
            }
            Some(object)
        } else {
            None
        }
    }

    /// Evacuate the object to another block using the `EvacAllocator`
    /// returning the new address or `None` if no evacuation was performed.
    ///
    /// An object is evacuated if it is not pinned, it resides on an
    /// evacuation candidate block and the evacuation allocator hat enough
    /// space left.
    ///
    /// On successful evacuation the old object is marked as forewarded an an
    /// forewarding pointer is installed.
    pub fn maybe_evacuate(&self, object: GCObjectRef) -> Option<GCObjectRef> {
        let block_info = unsafe { ImmixSpace::get_block_ptr(object) };
        let is_pinned = object.value().header().is_pinned();
        let is_candidate = unsafe { (*block_info).is_evacuation_candidate() };
        if is_pinned || !is_candidate {
            return None;
        }
        let size = object.size();
        if let Some(new_object) = self.evac_allocator.lock().allocate(size) {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    object.raw() as *const u8,
                    new_object.raw() as *mut u8,
                    size,
                );
                object.value_mut().header_mut().set_forwarded(object);
                //(*object).set_forwarded(new_object);
                self.set_gc_object(object);
            }
            log::debug!(
                "Evacuated object {:p} from block {:p} to {:p}",
                object.raw(),
                block_info,
                new_object.raw()
            );
            //valgrind_freelike!(object);
            return Some(new_object);
        }
        log::debug!(
            "Can't evacuation object {:p} from block {:p}",
            object.raw(),
            block_info
        );
        None
    }
    /// Extend the list of free blocks in the `EvacAllocator` for evacuation.
    pub fn extend_evac_headroom(&self, blocks: Vec<*mut BlockInfo>) {
        self.evac_allocator.lock().extend_evac_headroom(blocks);
    }
}

thread_local! {
    pub static LOCAL_ALLOCATOR: Arc<UnsafeCell<LocalAllocator>> = {
        let id = crate::VM.state.heap.immix_space.id.fetch_add(1,Ordering::AcqRel);
        let rc = crate::VM.state.heap.immix_space.block_allocator.clone();
        let rc = Arc::new(UnsafeCell::new(LocalAllocator::new(id,rc)));
        crate::VM.state.heap.immix_space.allocators.lock().push(rc.clone());
        rc
    };
}
