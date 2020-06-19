use std::sync::atomic::Ordering;
extern crate crossbeam_epoch as epoch;
use epoch::{Atomic, Owned, Shared};

pub struct Node<T: Copy> {
    value: T,
    next: *const Self,
}

pub struct LockFreeList<T: Copy> {
    next: Atomic<Node<T>>,
}
unsafe impl<T: Copy> Sync for LockFreeList<T> {}
unsafe impl<T: Copy> Send for LockFreeList<T> {}

impl<T: Copy> LockFreeList<T> {
    pub fn new() -> Self {
        Self {
            next: Atomic::null(),
        }
    }

    pub fn extend(&self, items: impl std::iter::Iterator<Item = T>) {
        for x in items {
            self.push(x);
        }
    }
    pub fn push(&self, val: T) {
        let guard = epoch::pin();

        let mut new = Owned::new(Node {
            value: val,
            next: std::ptr::null(),
        });
        loop {
            let old = self.next.load(Ordering::Relaxed, &guard);
            new.next = old.as_raw();
            match self
                .next
                .compare_and_set(old, new, Ordering::Release, &guard)
            {
                Ok(_) => break,
                Err(e) => {
                    new = e.new;
                }
            };
        }
    }

    pub fn pop(&self) -> Option<T> {
        let guard = epoch::pin();
        loop {
            let old = self.next.load(std::sync::atomic::Ordering::Acquire, &guard);

            match unsafe { old.as_ref() } {
                None => return None,
                Some(old2) => {
                    let next = old2.next;

                    if self
                        .next
                        .compare_and_set(old, Shared::from(next), Ordering::Release, &guard)
                        .is_ok()
                    {
                        unsafe {
                            guard.defer_destroy(old);
                        }
                        return Some(old2.value);
                    }
                    // spin_loop_hint();
                }
            }
        }
    }
}
impl<T: Copy> Drop for LockFreeList<T> {
    fn drop(&mut self) {
        let guard = epoch::pin();
        let mut next = self.next.load(Ordering::Relaxed, &guard).as_raw() as *mut Node<T>;
        while !next.is_null() {
            let n = unsafe { Owned::from_raw(next) };
            next = (*n).next as *mut Node<T>;
        }
    }
}
