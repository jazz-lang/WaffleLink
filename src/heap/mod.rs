pub mod object;
use object::*;
use std::collections::VecDeque;

pub const GC_WHITE: u8 = 0;
pub const GC_BLACK: u8 = 1;
pub const GC_GRAY: u8 = 2;

pub const GC_SWEEP: u8 = 0;
pub const GC_ROOTS: u8 = 1;
pub const GC_MARK: u8 = 2;
pub const GC_IDLE: u8 = 3;

pub struct TGC {
    eden: *mut GcBox<()>,
    eden_size: usize,
    eden_allowed_size: usize,
    roots: RootList,
    graystack: VecDeque<*mut GcBox<()>>,
    state: u8,
    white: u8,
    sweeps: *mut GcBox<()>,
    mi_heap: *mut libmimalloc_sys::mi_heap_t,
    stack_begin: *const usize,
    stack_end: *const usize,
}

pub const GC_NEW: u8 = 1;
pub const GC_OLD: u8 = 0;
pub const GC_OLD_REMEMBERED: u8 = 2;

const GC_VERBOSE_LOG: bool = true;
const GC_LOG: bool = true;

impl TGC {
    pub fn new(begin: *const usize) -> Self {
        Self {
            eden_allowed_size: 8 * 1024,
            eden_size: 0,
            eden: 0 as *mut _,
            roots: RootList::new(),
            graystack: Default::default(),
            state: GC_IDLE,
            white: GC_WHITE,
            sweeps: 0 as *mut _,
            mi_heap: unsafe { libmimalloc_sys::mi_heap_new() },
            stack_begin: begin,
            stack_end: begin,
        }
    }
    fn mark(&mut self, object: *mut GcBox<()>) {
        unsafe {
            let obj = &mut *object;
            if obj.header.tag() == GC_GRAY {
                obj.header.set_tag(GC_BLACK);
                obj.trait_object().visit_references(&mut |reference| {
                    self.mark_object(reference as *mut _);
                });
            }
        }
    }

    fn mark_object(&mut self, object: *mut GcBox<()>) {
        unsafe {
            let obj = &mut *object;
            if obj.header.tag() == GC_WHITE {
                if GC_VERBOSE_LOG && GC_LOG {
                    eprintln!("---mark {:p}", object);
                }
                obj.header.set_tag(GC_GRAY);
                self.graystack.push_back(object);
            }
        }
    }

    fn sweep(&mut self) {
        if GC_LOG {
            eprintln!("--begin sweep");
        }
        let mut previous = core::ptr::null_mut();
        let mut object = self.eden;
        let mut count = 0;
        let mut freed = 0;
        while object.is_null() == false {
            unsafe {
                let obj = &mut *object;
                if obj.header.tag() == GC_BLACK {
                    previous = object;
                    obj.header.set_tag(GC_WHITE);
                    object = obj.header.next().cast();
                } else {
                    let unreached = object;
                    object = obj.header.next().cast();

                    if previous.is_null() {
                        self.eden = object;
                    } else {
                        (&mut *previous).header.next = object;
                    }
                    let size = obj.trait_object().size() + core::mem::size_of::<Header>();
                    self.eden_size -= size;
                    freed += size;
                    if GC_LOG && GC_VERBOSE_LOG {
                        eprintln!("---sweep {:p} size={}", unreached, size);
                    }
                    core::ptr::drop_in_place(obj.trait_object());
                    libmimalloc_sys::mi_free(unreached.cast());
                }
                count += 1;
            }
        }
        if GC_LOG {
            eprintln!("--sweeped {} object(s) ({} bytes)", count, freed);
        }
    }

    fn mark_from_roots(&mut self) {
        if GC_LOG {
            eprintln!("--mark from roots");
        }

        /*unsafe {
            let mut scan = self.stack_begin;
            let mut end = self.stack_end;
            if scan > end {
                std::mem::swap(&mut scan, &mut end);
            }

            extern "C" {
                fn mi_is_in_heap_region(
                    //heap: *mut libmimalloc_sys::mi_heap_t,
                    p: *const u8,
                ) -> bool;
            }

            while scan < end {
                let ptr = scan.read() as *mut u8;
                eprintln!("try {:p}", ptr);
                if mi_is_in_heap_region(ptr)
                   // && libmimalloc_sys::mi_heap_check_owned(self.mi_heap, ptr.cast())
                    && libmimalloc_sys::mi_heap_contains_block(self.mi_heap, ptr.cast())
                {
                    if GC_LOG && GC_VERBOSE_LOG {
                        eprintln!("---conservative mark {:p} at {:p}", ptr, scan);
                    }
                    self.mark_object(ptr.cast());
                }
                scan = scan.offset(1);
            }
        }*/

        let this = unsafe { &mut *(self as *mut Self) };
        this.roots.walk(&mut |root| unsafe {
            self.mark_object((&*root).obj);
        });
        if GC_LOG {
            eprintln!("--process mark stack");
        }
        while let Some(item) = self.graystack.pop_front() {
            self.mark(item);
        }
    }
    #[inline(never)]
    pub fn collect_garbage(&mut self, stack_end: *const usize) {
        self.stack_end = stack_end;
        if GC_LOG {
            eprintln!(
                "--GC begin, heap size {}(threshold {})",
                self.eden_size, self.eden_allowed_size
            );
        }
        self.mark_from_roots();
        let before = self.eden_size;
        self.sweep();
        if self.eden_size >= self.eden_allowed_size {
            self.eden_allowed_size = (self.eden_size as f64 / 0.75) as usize;
        }
        if GC_LOG {
            eprintln!(
                "--GC end, threshold {}\n---heap size before GC={}\n---heap size after GC={}",
                self.eden_allowed_size, before, self.eden_size
            );
        }
    }

    pub fn allocate<T: GcObject>(&mut self, value: T) -> Root<T> {
        let mut mem = unsafe {
            libmimalloc_sys::mi_heap_malloc(
                self.mi_heap,
                value.size() + core::mem::size_of::<Header>(),
            )
        };
        unsafe {
            if self.eden_size > self.eden_allowed_size {
                let stack = 0usize;
                self.collect_garbage(&stack);
            }
            self.eden_size += value.size() + core::mem::size_of::<Header>();
            let mem = mem.cast::<GcBox<T>>();
            mem.write(GcBox {
                header: Header {
                    vtable: core::mem::transmute::<_, TraitObject>(&value as &dyn GcObject).vtable,
                    next: 0 as *mut _,
                },
                value,
            });
            (&mut *mem).header.set_tag(GC_WHITE);
            (&mut *mem).header.set_next(self.eden.cast());
            if GC_VERBOSE_LOG {
                eprintln!("--allocate {:p}", mem);
            }
            self.eden = mem.cast();
            self.roots.root(mem)
        }
    }
}
