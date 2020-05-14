use super::api::*;
use crate::common::Address;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum CellState {
    White,
    Grey,
    Black,
}

pub struct HeapInner<T: ?Sized + Trace + Finalizer> {
    pub(super) state: bool,
    pub(super) value: T,
}

impl<T: ?Sized + Trace> HeapInner<T> {
    pub(crate) fn set_fwdptr(&self, _: Address) {}
    pub(crate) fn fwdptr(&self) -> Address {
        Address::null()
    }
    pub(crate) fn is_marked(&self) -> bool {
        false
    }
    pub(crate) fn mark(&self, _: bool) {}
}
fn dealloc<T>(x: *mut T) {
    unsafe {
        std::alloc::dealloc(x as *mut u8, std::alloc::Layout::new::<T>());
    }
}

fn alloc<T>() -> *mut T {
    unsafe { std::alloc::alloc(std::alloc::Layout::new::<T>()).cast() }
}
pub struct Heap {
    heap: Vec<*mut HeapInner<dyn Trace>>,
    roots: Vec<*mut dyn RootedTrait>,
    threshold: usize,
    allocated: usize,
}
use std::collections::VecDeque;
impl Heap {
    pub fn new() -> Self {
        Self {
            heap: vec![],
            roots: vec![],
            threshold: 8 * 1024, // 8kb
            allocated: 0,
        }
    }
    fn sweep(&mut self) {
        self.heap.retain(|&item| unsafe {
            let i = &mut *item;
            if i.state {
                i.state = true;
                return true;
            } else {
                i.value.finalize();
                let _ = Box::from_raw(item);
                return false;
            }
        });
    }
    fn mark(&mut self) {
        let mut stack = VecDeque::new();
        self.mark_roots(&mut stack);
        self.trace(&mut stack);
    }

    fn mark_object(
        &mut self,
        x: &mut HeapInner<dyn Trace>,
        stack: &mut VecDeque<*mut HeapInner<dyn Trace>>,
    ) {
        if x.state {
            return;
        }
        x.state = true;
        stack.push_back(x);
    }

    fn trace(&mut self, stack: &mut VecDeque<*mut HeapInner<dyn Trace>>) {
        while let Some(x) = stack.pop_front() {
            let item = unsafe { &mut *x };
            item.value
                .references()
                .iter()
                .for_each(|x| self.mark_object(unsafe { &mut *(&**x).inner() }, stack));
        }
    }

    fn mark_roots(&mut self, stack: &mut VecDeque<*mut HeapInner<dyn Trace>>) {
        /*self.roots.retain(|item| unsafe {
            let r = &mut **item;
            if r.is_rooted() {
                self.mark_object(&mut *r.inner(), stack);
                return true;
            } else {
                let _ = Box::from_raw(r);
                false
            }
        });*/
        let mut roots = vec![];
        unsafe {
            while let Some(item) = self.roots.pop() {
                let r = &mut *item;
                log::warn!("Root {:p}", item);
                if r.is_rooted() {
                    self.mark_object(&mut *r.inner(), stack);
                    roots.push(item);
                } else {
                    log::warn!("Unroot {:p}", item);
                    let _ = Box::from_raw(r);
                    continue;
                }
            }
        }
        std::mem::swap(&mut self.roots, &mut roots);
    }

    pub fn collect(&mut self) {
        self.mark();
        self.sweep();
    }
    pub fn root<T: Trace + Sized + 'static>(&mut self, x: Handle<T>) -> Rooted<T> {
        let root = Box::into_raw(Box::new(RootedInner {
            counter: 1,
            inner: x.inner,
        }));
        self.roots.push(root);
        return Rooted { inner: root };
    }
    pub fn allocate<T: Trace + Sized + 'static>(&mut self, value: T) -> Rooted<T> {
        //self.safepoint();
        let ptr = Box::into_raw(Box::new(HeapInner {
            state: false,
            value,
        }));
        log::trace!(
            "Allocate {} bytes,total: {} bytes",
            std::mem::size_of::<HeapInner<T>>(),
            self.allocated + std::mem::size_of::<HeapInner<T>>()
        );
        unsafe {
            let root = Box::into_raw(Box::new(RootedInner {
                counter: 1,
                inner: ptr,
            }));
            self.allocated += std::mem::size_of::<HeapInner<T>>();
            self.heap.push(ptr);
            self.roots.push(root);
            return Rooted { inner: root };
        }
        unimplemented!()
    }
    pub fn safepoint(&mut self) {
        if self.threshold <= self.allocated {
            log::trace!(
                "Collecting, threshold is {} bytes and {} bytes allocated",
                self.threshold,
                self.allocated
            );
            self.collect();
            if self.allocated >= self.threshold {
                self.threshold = (self.allocated as f64 / 0.7) as usize;
            }
        }
    }
}
