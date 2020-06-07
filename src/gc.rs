use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub trait Collectable {
    fn walk_references(&self, trace: &mut dyn FnMut(*const Handle<dyn Collectable>));
}

pub struct GcHeader<T: Collectable + ?Sized> {
    pub(crate) mark_bit: AtomicBool,
    pub(crate) value: T,
}
pub struct Handle<T: Collectable + ?Sized> {
    inner: std::ptr::NonNull<GcHeader<T>>,
}

impl<T: Collectable + ?Sized> Handle<T> {
    pub fn get(&self) -> &mut T {
        unsafe { &mut (&mut *self.inner.as_ptr()).value }
    }
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut (&mut *self.inner.as_ptr()).value }
    }
}

impl<T: Collectable + ?Sized + Eq> Eq for Handle<T> {}
impl<T: Collectable + ?Sized + PartialEq> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: Collectable + ?Sized + std::hash::Hash> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}
impl std::borrow::Borrow<str> for Handle<String> {
    fn borrow(&self) -> &str {
        self.get()
    }
}

use std::ops::{Deref, DerefMut};

impl<T: Collectable + ?Sized> Deref for Handle<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}

impl<T: Collectable + ?Sized> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Collectable + ?Sized> Copy for Handle<T> {}
impl<T: Collectable + ?Sized> DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.get()
    }
}
pub struct Root<T: Collectable + ?Sized> {
    inner: *mut RootInner<T>,
}

impl<T: Collectable + ?Sized> Root<T> {
    pub(crate) fn inner(&self) -> &mut RootInner<T> {
        debug_assert!(!self.inner.is_null());
        unsafe { &mut *self.inner }
    }

    pub fn to_heap(&self) -> Handle<T> {
        self.inner().handle
    }
}

pub struct RootInner<T: Collectable + ?Sized> {
    pub(crate) handle: Handle<T>,
    pub(crate) refcount: AtomicUsize,
}

impl<T: Collectable + ?Sized> Collectable for Handle<T> {
    fn walk_references(&self, trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        trace(self as *const Self as *const Handle<dyn Collectable>);
    }
}

impl<T: Collectable + ?Sized> Drop for Root<T> {
    fn drop(&mut self) {
        let inner = self.inner();
        inner.refcount.fetch_sub(1, Ordering::AcqRel);
    }
}

macro_rules! simple_gc {
    ($($t: ty)*) => {
        $(
            impl Collectable for $t {
                fn walk_references(&self,_: &mut dyn FnMut(*const Handle<dyn Collectable>)) {}
            }
        )*
    };
}

simple_gc!(
    u8
    i8
    i16
    u16
    i32
    u32
    i64
    u64
    i128
    u128
    String
    std::fs::File
);

impl<T: Collectable> Collectable for Vec<T> {
    fn walk_references(&self, trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        for item in self.iter() {
            item.walk_references(trace);
        }
    }
}

pub struct WaffleHeap {
    lock: parking_lot::Mutex<Vec<*mut GcHeader<dyn Collectable>>>,
    allocated: AtomicUsize,
    threshold: AtomicUsize,
}
use super::*;
impl WaffleHeap {
    pub fn new() -> Self {
        Self {
            lock: parking_lot::Mutex::new(vec![]),
            allocated: AtomicUsize::new(0),
            threshold: AtomicUsize::new(16 * 1024),
        }
    }
    pub fn collect(&self, vm: &Machine) {
        vm.threads.stop_the_world(|threads| {
            let mut lock = self.lock.lock();
            let mut stack = std::collections::LinkedList::new();

            for thread in threads.iter() {
                thread.roots(|item| unsafe {
                    let item = item as *mut Handle<dyn Collectable>;
                    let r = &mut *item;
                    let x = r.inner.as_mut();
                    if !x.mark_bit.load(Ordering::Relaxed) {
                        stack.push_back(item);
                        x.mark_bit.store(true, Ordering::Relaxed);
                    }
                });
            }

            while let Some(item) = stack.pop_front() {
                unsafe {
                    let r = &mut *item;
                    let x = r.inner.as_mut();

                    x.value.walk_references(&mut |elem| {
                        let elem = elem as *mut Handle<dyn Collectable>;
                        let r = &mut *elem;
                        let x = r.inner.as_mut();
                        if x.mark_bit.load(Ordering::Relaxed) {
                            return;
                        }
                        x.mark_bit.store(true, Ordering::Relaxed);
                        stack.push_back(elem);
                    })
                }
            }

            unsafe {
                lock.retain(|item| {
                    let item_ref = &mut **item;
                    if item_ref.mark_bit.load(Ordering::Relaxed) {
                        item_ref.mark_bit.store(false, Ordering::Relaxed);
                        true
                    } else {
                        std::ptr::drop_in_place(*item);
                        mi_free((*item) as *mut u8);
                        false
                    }
                })
            }

            let a = self.allocated.load(Ordering::Relaxed);

            if a >= self.threshold.load(Ordering::Relaxed) {
                self.threshold
                    .store((a as f64 * 0.75) as usize, Ordering::Relaxed);
            }
        });
    }

    pub fn allocate<T: Collectable + Sized + 'static>(&self, vm: &Machine, val: T) -> Root<T> {
        if self.allocated.load(Ordering::Acquire) >= self.threshold.load(Ordering::Relaxed) {
            self.collect(vm);
        }

        let thread = threads::THREAD.with(|item| item.borrow().clone());
        unsafe {
            let mem = mi_malloc(std::mem::size_of::<GcHeader<T>>()) as *mut GcHeader<T>;
            mem.write(GcHeader {
                mark_bit: AtomicBool::new(false),
                value: val,
            });
            self.allocated
                .fetch_add(std::mem::size_of::<GcHeader<T>>(), Ordering::AcqRel);
            let mut lock = self.lock.lock();
            lock.push(mem);
            drop(lock);
            let root: Root<T> = Root {
                inner: Box::into_raw(Box::new(RootInner {
                    handle: Handle {
                        inner: std::ptr::NonNull::new_unchecked(mem),
                    },
                    refcount: AtomicUsize::new(1),
                })),
            };
            thread.roots.borrow_mut().push(root.inner as *mut _);
            root
        }
    }
}
unsafe impl Send for WaffleHeap {}
#[repr(C)]
pub struct MiHeap {
    _field: [u8; 0],
}

#[link(name = "mimalloc")]
extern "C" {
    fn mi_malloc(size: usize) -> *mut u8;
    fn mi_free(ptr: *mut u8);
}
