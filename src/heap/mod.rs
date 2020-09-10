pub mod block_directory;
pub mod block_directory_bits;
pub mod freelist;
pub mod local_allocator;
pub mod markedblock;
pub mod object;
use object::*;
use std::collections::VecDeque;

pub const GC_WHITE: u8 = 0;
pub const GC_BLACK: u8 = 1;
pub const GC_GRAY: u8 = 2;
pub const GC_NEW: u8 = 0;
pub const GC_OLD: u8 = 1;
pub const GC_NONE: u8 = 2;
pub struct TGC {
    eden: *mut GcBox<()>,
    eden_size: usize,
    eden_allowed_size: usize,
    old: *mut GcBox<()>,
    old_size: usize,
    old_allowed_size: usize,
    roots: RootList,
    gc_ty: u8,
    graystack: VecDeque<*mut GcBox<()>>,
    blacklist: Vec<*mut GcBox<()>>,
    mi_heap: *mut libmimalloc_sys::mi_heap_t,
    stack_begin: *const usize,
    stack_end: *const usize,
}

const GC_VERBOSE_LOG: bool = true;
const GC_LOG: bool = true;

impl TGC {
    pub fn new(begin: *const usize) -> Self {
        Self {
            eden_allowed_size: 8 * 1024,
            eden_size: 0,
            eden: 0 as *mut _,
            old: 0 as *mut _,
            old_allowed_size: 16 * 1024,
            old_size: 0,
            gc_ty: GC_NONE,
            roots: RootList::new(),
            graystack: Default::default(),
            blacklist: Vec::new(),
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
        //panic!();
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

    fn sweep(&mut self, eden: bool) {
        let mut count = 0;
        let mut freed = 0;
        if eden {
            if GC_LOG {
                eprintln!("--begin eden sweep");
            }

            let mut object = self.eden;
            self.eden = 0 as *mut _;
            while object.is_null() == false {
                unsafe {
                    let obj = &mut *object;
                    let next = obj.header.next();
                    let size = obj.trait_object().size() + core::mem::size_of::<Header>();
                    self.eden_size -= size;
                    if obj.header.tag() == 1 {
                        obj.header.set_tag(0);
                        obj.header.set_next(self.old.cast());
                        self.old = object;
                        if GC_LOG && GC_VERBOSE_LOG {
                            eprintln!("---promote {:p}", obj);
                        }
                        obj.header.set_next_tag(1);
                        self.old_size += obj.trait_object().size() + core::mem::size_of::<Header>();
                    } else {
                        let unreached = object;

                        freed += size;
                        if GC_LOG && GC_VERBOSE_LOG {
                            eprintln!("---sweep {:p} size={}", unreached, size);
                        }
                        core::ptr::drop_in_place(obj.trait_object());
                        libmimalloc_sys::mi_free(unreached.cast());
                        count += 1;
                    }
                    object = next.cast();
                }
            }
            self.eden = core::ptr::null_mut();
        } else {
            let mut previous = core::ptr::null_mut();
            let mut object = self.old;

            while object.is_null() == false {
                unsafe {
                    let obj = &mut *object;
                    if obj.header.tag() == 1 {
                        previous = object;
                        obj.header.set_tag(0);
                        object = obj.header.next().cast();
                    } else {
                        let unreached = object;
                        object = obj.header.next().cast();

                        if previous.is_null() {
                            self.old = object;
                        } else {
                            (&mut *previous).header.next = object;
                        }
                        let size = obj.trait_object().size() + core::mem::size_of::<Header>();
                        self.old_size -= size;
                        freed += size;
                        if GC_LOG && GC_VERBOSE_LOG {
                            eprintln!("---sweep {:p} size={}", unreached, size);
                        }
                        core::ptr::drop_in_place(obj.trait_object());
                        libmimalloc_sys::mi_free(unreached.cast());
                        count += 1;
                    }
                }
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

        let this = unsafe { &mut *(self as *mut Self) };
        this.roots.walk(&mut |root| unsafe {
            self.graystack.push_back((&*root).obj);
        });
        if GC_LOG {
            eprintln!("--process gray stack");
        }
        self.process_gray();
    }
    #[inline(never)]
    pub fn collect_garbage(&mut self, stack_end: *const usize) {
        if self.gc_ty == GC_NONE {
            self.gc_ty = GC_NEW;
        }
        self.stack_end = stack_end;
        if GC_LOG {
            eprintln!(
                "--GC begin, heap size {}(threshold {})",
                self.eden_size, self.eden_allowed_size
            );
        }
        self.mark_from_roots();
        let before = self.eden_size;
        self.sweep(self.gc_ty == GC_NEW);

        if self.old_size >= self.old_allowed_size {
            self.gc_ty = GC_OLD;
            self.collect_garbage(stack_end);
        }
        while let Some(obj) = self.blacklist.pop() {
            unsafe {
                (&mut *obj).header.set_tag(0);
            }
        }
        if self.eden_size >= self.eden_allowed_size {
            self.eden_allowed_size = (self.eden_size as f64 / 0.75) as usize;
        }
        if self.old_size >= self.old_allowed_size {
            self.old_allowed_size = (self.old_size as f64 / 0.5) as usize;
        }
        if GC_LOG {
            eprintln!(
                "--GC end, threshold {}\n---eden heap size before GC={}\n---eden heap size after GC={}",
                self.eden_allowed_size, before, self.eden_size
            );
        }
        self.eden = 0 as *mut _;
        self.gc_ty = GC_NONE;
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
                self.gc_ty = GC_NEW;
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
            (&mut *mem).header.set_tag(0);
            (&mut *mem).header.set_next(self.eden.cast());
            (&mut *mem).header.set_next_tag(0);
            if GC_VERBOSE_LOG {
                eprintln!("--allocate {:p}", mem);
            }
            self.eden = mem.cast();
            self.roots.root(mem)
        }
    }
    fn in_current_space(&self, val: &GcBox<()>) -> bool {
        unsafe {
            if self.gc_ty == GC_OLD {
                val.header.next_tag() == 1
            } else {
                val.header.next_tag() == 0
            }
        }
    }
    fn visit_value(&mut self, obj: &mut GcBox<()>) {
        obj.trait_object().visit_references(&mut |object| {
            self.graystack.push_back(object as *mut _);
        })
    }
    fn process_gray(&mut self) {
        while let Some(item) = self.graystack.pop_front() {
            unsafe {
                let obj = &mut *item;
                if obj.header.tag() < 1 {
                    if !self.in_current_space(obj) {
                        if obj.header.tag() != 2 {
                            if GC_VERBOSE_LOG && GC_LOG {
                                eprintln!("---blacklist {:p}", obj);
                            }
                            obj.header.set_tag(2);
                            self.blacklist.push(item);
                            self.visit_value(obj);
                        }
                        continue;
                    }
                    if GC_VERBOSE_LOG && GC_LOG {
                        eprintln!("---mark {:p}", obj);
                    }
                    obj.header.set_tag(1);
                    self.visit_value(obj);
                }
            }
        }
    }
}
