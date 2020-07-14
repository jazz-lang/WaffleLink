use crate::gc::*;
use crate::object::*;
pub mod block;

pub const SIZE_CLASS_1: usize = 16;
pub const SIZE_CLASS_2: usize = 32;
pub const SIZE_CLASS_3: usize = 48;
pub const SIZE_CLASS_4: usize = 64;
pub const SIZE_CLASS_5: usize = 128;
pub const SIZE_CLASS_6: usize = 512;
pub const LARGE_SIZE: usize = 7;
pub const SIZE_CLASSES: usize = 6;

pub struct Heap {
    pub size_classes: [Vec<*mut block::HeapBlock>; SIZE_CLASSES],
    pub start: *mut u8,
}

impl Heap {
    pub fn new(stack_start: *const bool) -> Self {
        println!("{:p}", stack_start);
        Self {
            start: stack_start as *mut u8,
            size_classes: [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],
        }
    }
    pub fn size_class_for(size: usize) -> usize {
        match size {
            x if x <= SIZE_CLASS_1 => 0,
            x if x <= SIZE_CLASS_2 => 1,
            x if x <= SIZE_CLASS_3 => 2,
            x if x <= SIZE_CLASS_4 => 3,
            x if x <= SIZE_CLASS_5 => 4,
            x if x <= SIZE_CLASS_6 => 5,
            _ => LARGE_SIZE,
        }
    }
    pub fn size_class_size_for(x: usize) -> usize {
        match x {
            0 => SIZE_CLASS_1,
            1 => SIZE_CLASS_2,
            2 => SIZE_CLASS_3,
            3 => SIZE_CLASS_4,
            4 => SIZE_CLASS_5,
            5 => SIZE_CLASS_6,
            _ => unreachable!(),
        }
    }
    pub fn allocate(&mut self, size: usize) -> Address {
        let sc = Self::size_class_for(size);
        if sc == LARGE_SIZE {
            println!("{}", size);
            todo!("Large classes is not yet supported")
        }
        log::trace!("Allocate {} bytes in size class #{}", size, sc);
        unsafe {
            for block in self.size_classes[sc].iter_mut() {
                let mem = (&mut **block).allocate();
                if mem.is_non_null() {
                    return mem;
                }
            }
            self.size_classes[sc].push(block::HeapBlock::new(Self::size_class_size_for(sc)));
            (&mut **self.size_classes[sc].last_mut().unwrap()).allocate()
        }
    }

    fn get_heap_block(object: Address) -> *mut block::HeapBlock {
        let off = object.to_usize() % block::HeapBlock::BLOCK_SIZE;
        (object.to_usize() as isize + (-(off as isize))) as *mut _
    }

    pub fn collect(&mut self, sp: Address) {
        let filter = |frame_addr: Address| unsafe {
            if frame_addr.is_non_null() {
                let value_pointer = *frame_addr.to_mut_ptr::<*mut u8>();
                if value_pointer.is_null() {
                    return false;
                }
                let block = Self::get_heap_block(Address::from_ptr(value_pointer));
                for sc in self.size_classes.iter() {
                    if sc.iter().any(|b| {
                        let bptr = &**b as *const block::HeapBlock;
                        bptr == block
                    }) {
                        return true;
                    }
                }
                false
            } else {
                false
            }
        };
        let mut mark_stack = vec![];
        // conservative roots
        {
            let mut start = sp;
            let mut end = Address::from_ptr(self.start);
            if start > end {
                // if stack grows from bottom to top then swap
                std::mem::swap(&mut start, &mut end);
            }
            if start.to_usize() % 8 != 0 {
                start = Address::from(start.to_usize() + 8 - start.to_usize() % 8);
            }
            if end.to_usize() % 8 != 0 {
                end = Address::from(end.to_usize() + 8 - end.to_usize() % 8);
            }
            log::debug!(
                "Scan for conservative roots in range {:p}..{:p}",
                start.to_ptr::<u8>(),
                end.to_ptr::<u8>()
            );
            while start < end {
                //println!("{:p}", start.to_mut_ptr::<u8>());
                unsafe {
                    let frame = start;
                    if filter(frame) {
                        let block =
                            Self::get_heap_block(Address::from_ptr(*frame.to_mut_ptr::<*mut u8>()));
                        if (&*block).is_marked(*start.to_mut_ptr::<Address>()) {
                            let mut cell: Ref<Obj> = Ref {
                                ptr: (*start.to_mut_ptr::<Address>()).to_mut_ptr(),
                            };
                            log::trace!(
                                "Found GC pointer {:p} at {:p}",
                                (*start.to_mut_ptr::<Address>()).to_ptr::<u8>(),
                                start.to_ptr::<u8>(),
                            );
                            cell.header_mut().mark_non_atomic();
                            mark_stack.push(*start.to_mut_ptr::<Address>());
                            start = start.add_ptr(1);
                            continue;
                        }
                    }
                }
                start = start.add_ptr(1);
            }
        }
        // marking
        {
            while let Some(cell_addr) = mark_stack.pop() {
                let mut cell: Ref<Obj> = Ref {
                    ptr: cell_addr.to_mut_ptr(),
                };

                cell.header_mut().mark_non_atomic();
                debug_assert!(cell.header().vtblptr().is_non_null());
                if let Some(trace) = cell.header().vtbl().trace_fn {
                    trace(cell, &mut |mut object| {
                        if object.header().is_marked_non_atomic() {
                            return;
                        }
                        object.header_mut().mark_non_atomic();
                        mark_stack.push(Address::from_ptr(object.raw()));
                    });
                }
            }
        }

        // sweeping
        unsafe {
            for sc in self.size_classes.iter_mut() {
                let len = sc.len();
                let mut del = 0;
                {
                    for i in 0..len {
                        if (&mut *sc[i]).sweep() {
                            del += 1;
                            std::ptr::drop_in_place(sc[i]);
                            std::alloc::dealloc(
                                sc[i].cast(),
                                std::alloc::Layout::from_size_align_unchecked(16 * 1024, 16 * 1024),
                            );
                        } else if del > 0 {
                            sc.swap(i - del, i);
                        }
                    }
                }
                if del > 0 {
                    sc.truncate(len - del);
                }
            }
        }
    }
}
