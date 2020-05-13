use super::api::*;
use crate::common::Address;

#[derive(Copy,Clone,PartialEq,Eq)]
#[repr(u8)]
pub enum CellState {
    White,
    Grey,
    Black
}

pub struct HeapInner<T: ?Sized + Trace + Finalizer> {
    pub(super) state: CellState,
    pub(super) value: T
}

impl<T: ?Sized + Trace> HeapInner<T> {
    pub(crate) fn set_fwdptr(&self,_: Address) {}
    pub(crate) fn fwdptr(&self) -> Address {
        Address::null()
    }
    pub(crate) fn is_marked(&self) -> bool {
        false
    }
    pub(crate) fn mark(&self,_: bool) {}
}
fn dealloc<T>(x: *mut T) {
    unsafe {
        std::alloc::dealloc(x as *mut u8,std::alloc::Layout::new::<T>());
    }
}

fn alloc<T> () -> *mut T {
    unsafe {
        std::alloc::alloc(std::alloc::Layout::new::<T>()).cast()
    }
}
pub struct Heap {

    heap: Vec<*mut HeapInner<dyn Trace>>,
    roots: Vec<*mut dyn RootedTrait>,
    threshold: usize,
    allocated: usize,
}

impl Heap {
    pub fn new() -> Self {
        Self {

            heap: vec![],
            roots: vec![],
            threshold: 8 * 1024, // 8kb
            allocated: 0

        }
    }

    fn sweep(&mut self) {
        let mut n = self.allocated;

        self.heap.retain(|item| unsafe {
            let item = &mut **item;
            if let CellState::White = item.state {
                n -= std::mem::size_of_val(item);
                /*std::ptr::drop_in_place(item);
                dealloc(item);
                */
                let _ = Box::from_raw(item);
                return false;

            }
            item.state = CellState::White;
            return true;
        });
    }
    fn mark(&mut self) {
        let mut worklist = vec![];
        self.roots.retain(|&item| unsafe {
            let r = &mut *item;
            if r.is_rooted() {
                (&mut *r.inner()).state = CellState::Grey;
                worklist.push(r.inner());
                true
            } else {
                let _ = Box::from_raw(item);
                false
            }
        });

        while let Some(item) = worklist.pop() {
            unsafe {

                let item_ref = &mut *item;
                if item_ref.state == CellState::Black {
                    continue;
                }
                let mut tracer = Tracer::default();
                /*item_ref.value.trace_with(&mut tracer);
                tracer.for_each(|item| {
                    let item = (&*item).inner();
                    (&mut *item).state = CellState::Grey;
                    worklist.push(item);
                });*/
                item_ref.value.references().iter().for_each(|&item| {
                    let item = (&*item).inner();
                    (&mut *item).state = CellState::Grey;
                    worklist.push(item);
                });
                item_ref.state = CellState::Black;
            }
        }
    }

    pub fn collect(&mut self) {
        self.mark();
        self.sweep();

    }
    pub fn root<T: Trace + Sized + 'static>(&mut self,x: Handle<T>) -> Rooted<T> {
        let root = Box::into_raw(Box::new(RootedInner {
            counter: 1,
            inner: x.inner
        }));
        self.roots.push(root);
        return Rooted {
            inner: root
        }
    }
    pub fn allocate<T: Trace + Sized + 'static>(&mut self,value: T) -> Rooted<T> {
        let ptr = alloc::<HeapInner<T>>();
        unsafe {
            ptr.write(HeapInner {
                state: CellState::White,
                value
            });
            let root = Box::into_raw(Box::new(RootedInner {
                counter: 1,
                inner: ptr
            }));
            self.allocated += std::mem::size_of::<HeapInner<T>>();
            self.heap.push(ptr);
            self.roots.push(root);
            return Rooted {
                inner: root
            }
        }
        unimplemented!()
    }
    pub fn safepoint(&mut self) {
        if self.threshold <= self.allocated {
            self.collect();
            if self.allocated >= self.threshold {
                self.threshold = (self.allocated as f64 / 0.7) as usize;
            }
        }
    }
}