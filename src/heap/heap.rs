use super::api::*;
use crate::common::Address;
use crate::runtime::*;
use cell::*;
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum CellState {
    White,
    Grey,
    Black,
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
    heap: Vec<CellPointer>,

    threshold: usize,
    allocated: usize,
}

pub struct HeapCell {
    pub marked: bool,
    pub cell: CellPointer
}

use std::collections::VecDeque;
impl Heap {
    pub fn new() -> Self {
        Self {
            heap: vec![],
            threshold: 8 * 1024, // 8kb
            allocated: 0,
        }
    }
    /* fn sweep(&mut self) {
            let mut n = self.allocated;
            self.heap.retain(|&item| unsafe {
                let i = &mut *item;
                if i.state {
                    i.state = true;
                    return true;
                } else {
                    n -= std::mem::size_of_val(i);
                    i.value.finalize();
                    let _ = Box::from_raw(item);
                    return false;
                }
            });
            self.allocated = n;
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
            \
        }

        pub fn collect(&mut self) {
            self.mark();
            self.sweep();
        }
    */
    pub fn allocate(&mut self, cell: Cell) -> CellPointer {
        //self.safepoint();
        let ptr = Box::into_raw(Box::new(cell));

        unsafe {

            self.allocated += std::mem::size_of::<Cell>();
            self.heap.push(CellPointer { raw: ptr });
            return CellPointer { raw: ptr };
        }
        unimplemented!()
    }

    pub fn safepoint(&mut self) -> bool {
        self.threshold <= self.allocated
    }
}

pub struct Collection<'a> {
    rt: &'a mut Runtime,
    stack: VecDeque<*const CellPointer>,
}

impl<'a> Collection<'a> {
    fn mark_obj(&mut self,obj: *const CellPointer) {
        unsafe {
            if obj.is_null() {
                return;
            }
            let obj = obj as *mut CellPointer;
            let obj_ref = &mut *obj;
            if obj_ref.marked {
                return;
            }
            obj_ref.marked = true;
            self.stack.push_back(obj);
        }
    }

    fn mark(&mut self) {
        while let Some(object) = self.stack.pop_front() {
            unsafe {
                let object_ref = &mut *(object as *mut CellPointer);
                if object_ref.marked {
                    continue;
                }
                object_ref.marked = true;
                object_ref.each_pointer(&mut self.stack);
            }
        }
    }

    fn sweep(&mut self) {
        self.rt.heap.heap.retain(|item| {
            if !item.marked {
                unsafe {
                    let _ = Box::from_raw(item.raw);
                }
                return false;
            } else {
                item.get_mut().marked = false;
                true
            }
        })
    }
    pub fn run(rt: &'a mut Runtime) {
        let mut rootset = Default::default();
        rt.each_pointer(&mut rootset);
        let mut this = Self {
            rt,
            stack: rootset
        };

        this.mark();
        this.sweep();
        if this.rt.heap.allocated >= this.rt.heap.threshold {
            this.rt.heap.threshold = (this.rt.heap.allocated as f64 / 0.7) as usize;
        }


    }


}
