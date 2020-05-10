use crate::common::*;
use std::cell::RefCell;
use std::convert::From;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct ThreadLocalData {
    tlab_top: AtomicUsize,
    tlab_end: AtomicUsize,
    concurrent_marking: AtomicBool,
    guard_stack_limit: AtomicUsize,
    real_stack_limit: AtomicUsize,
    dtn: AtomicUsize,
}

impl ThreadLocalData {
    pub fn new() -> ThreadLocalData {
        ThreadLocalData {
            tlab_top: AtomicUsize::new(0),
            tlab_end: AtomicUsize::new(0),
            concurrent_marking: AtomicBool::new(false),
            guard_stack_limit: AtomicUsize::new(0),
            real_stack_limit: AtomicUsize::new(0),
            dtn: AtomicUsize::new(0),
        }
    }

    pub fn tlab_initialize(&self, start: Address, end: Address) {
        assert!(start <= end);

        self.tlab_top.store(start.to_usize(), Ordering::Relaxed);
        self.tlab_end.store(end.to_usize(), Ordering::Relaxed);
    }

    pub fn tlab_rest(&self) -> usize {
        let tlab_top = self.tlab_top.load(Ordering::Relaxed);
        let tlab_end = self.tlab_end.load(Ordering::Relaxed);

        tlab_end - tlab_top
    }

    pub fn tlab_region(&self) -> Region {
        let tlab_top = self.tlab_top.load(Ordering::Relaxed);
        let tlab_end = self.tlab_end.load(Ordering::Relaxed);

        Region::new(tlab_top.into(), tlab_end.into())
    }

    pub fn set_stack_limit(&self, stack_limit: Address) {
        self.guard_stack_limit
            .store(stack_limit.to_usize(), Ordering::Relaxed);
        self.real_stack_limit
            .store(stack_limit.to_usize(), Ordering::Relaxed);
    }

    pub fn tlab_top_offset() -> i32 {
        offset_of!(ThreadLocalData, tlab_top) as i32
    }

    pub fn tlab_end_offset() -> i32 {
        offset_of!(ThreadLocalData, tlab_end) as i32
    }

    pub fn concurrent_marking_offset() -> i32 {
        offset_of!(ThreadLocalData, concurrent_marking) as i32
    }

    pub fn guard_stack_limit_offset() -> i32 {
        offset_of!(ThreadLocalData, guard_stack_limit) as i32
    }

    pub fn real_stack_limit(&self) -> Address {
        Address::from(self.real_stack_limit.load(Ordering::Relaxed))
    }

    pub fn real_stack_limit_offset() -> i32 {
        offset_of!(ThreadLocalData, real_stack_limit) as i32
    }

    pub fn dtn_offset() -> i32 {
        offset_of!(ThreadLocalData, dtn) as i32
    }

    pub fn arm_stack_guard(&self) {
        self.guard_stack_limit.store(!0, Ordering::Release);
    }

    pub fn unarm_stack_guard(&self) {
        let limit = self.real_stack_limit.load(Ordering::Relaxed);
        self.guard_stack_limit.store(limit, Ordering::Release);
    }
}
