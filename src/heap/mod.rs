//! # CakeGC
//!
//! Generational Mark&Sweep garbage collector.
//!
//!
//!
//!
//!
//!
//!

// all these modules is for future use in incremental GC
pub mod bitmap;
pub mod block_directory;
pub mod block_directory_bits;
pub mod freelist;
pub mod local_allocator;
pub mod markedblock;
pub mod markedspace;
pub mod precise_allocation;

pub mod object;
#[cfg(feature = "pmarking")]
pub mod pmarking;
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

    #[cfg(feature = "pmarking")]
    pool: scoped_threadpool::Pool,
    #[cfg(feature = "pmarking")]
    num_cpus: usize,
    pmark: bool,
}

pub const GC_VERBOSE_LOG: bool = true;
pub const GC_LOG: bool = true;

impl TGC {
    pub fn new(begin: *const usize, n_cpus: Option<usize>, pmark: bool) -> Self {
        Self {
            #[cfg(feature = "pmarking")]
            num_cpus: n_cpus.unwrap_or(num_cpus::get() / 2),
            #[cfg(feature = "pmarking")]
            pool: scoped_threadpool::Pool::new(n_cpus.unwrap_or(num_cpus::get() / 2) as u32),
            eden_allowed_size: 8 * 1024,
            eden_size: 0,
            eden: 0 as *mut _,
            old: 0 as *mut _,
            old_allowed_size: 16 * 1024,
            old_size: 0,
            pmark,
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
        #[cfg(feature = "pmarking")]
        {
            if self.pmark {
                if GC_LOG {
                    eprintln!(
                        "--start parallel marking\n---thread count={}",
                        self.pool.thread_count()
                    );
                    let mut roots = vec![];
                    self.roots.walk(&mut |root| unsafe {
                        roots.push(Address::from_ptr((&*root).obj));
                    });

                    let mut count = 0;
                    let blacklist_recv = pmarking::start(&roots, &mut self.pool, self.gc_ty);
                    if GC_LOG {
                        eprintln!("---pmarking finished, processing blacklist");
                    }
                    while let Ok(addr) = blacklist_recv.recv() {
                        count += 1;
                        if GC_LOG {
                            eprintln!("---received blacklisted object {}", addr);
                        }
                        unsafe {
                            addr.to_mut_obj().header.set_tag(0);
                        }
                    }
                    if GC_LOG {
                        if count == 0 {
                            eprintln!("---No objects from old gen was blacklisted");
                        }
                        eprintln!("--marking finished")
                    }
                }
            } else {
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
        }
        #[cfg(not(feature = "pmarking"))]
        {
            if self.pmark {
                panic!("WaffleLink compiled without support for parallel marking!");
            }
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

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Address(usize);

impl Address {
    #[inline(always)]
    pub fn from(val: usize) -> Address {
        Address(val)
    }

    #[inline(always)]
    pub fn offset_from(self, base: Address) -> usize {
        debug_assert!(self >= base);

        self.to_usize() - base.to_usize()
    }

    #[inline(always)]
    pub fn offset(self, offset: usize) -> Address {
        Address(self.0 + offset)
    }

    #[inline(always)]
    pub fn sub(self, offset: usize) -> Address {
        Address(self.0 - offset)
    }

    #[inline(always)]
    pub fn add_ptr(self, words: usize) -> Address {
        Address(self.0 + words * core::mem::size_of::<usize>())
    }

    #[inline(always)]
    pub fn sub_ptr(self, words: usize) -> Address {
        Address(self.0 - words * core::mem::size_of::<usize>())
    }

    #[inline(always)]
    pub fn to_mut_obj(self) -> &'static mut GcBox<()> {
        unsafe { &mut *self.to_mut_ptr::<_>() }
    }

    #[inline(always)]
    pub fn to_obj(self) -> &'static GcBox<()> {
        unsafe { &*self.to_mut_ptr::<_>() }
    }

    #[inline(always)]
    pub fn to_usize(self) -> usize {
        self.0
    }

    #[inline(always)]
    pub fn from_ptr<T>(ptr: *const T) -> Address {
        Address(ptr as usize)
    }

    #[inline(always)]
    pub fn to_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    #[inline(always)]
    pub fn to_mut_ptr<T>(&self) -> *mut T {
        self.0 as *const T as *mut T
    }

    #[inline(always)]
    pub fn null() -> Address {
        Address(0)
    }

    #[inline(always)]
    pub fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub fn is_non_null(self) -> bool {
        self.0 != 0
    }
}
use std::cmp::Ordering;
use std::fmt;
impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:x}", self.to_usize())
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:x}", self.to_usize())
    }
}

impl PartialOrd for Address {
    fn partial_cmp(&self, other: &Address) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Address {
    fn cmp(&self, other: &Address) -> Ordering {
        self.to_usize().cmp(&other.to_usize())
    }
}

impl From<usize> for Address {
    fn from(val: usize) -> Address {
        Address(val)
    }
}

pub const fn round_up_to_multiple_of(divisor: usize, x: usize) -> usize {
    (x + (divisor - 1)) & !(divisor - 1)
}

pub struct Heap {
    pub(crate) object_space: markedspace::MarkedSpace,
}
impl Heap {
    pub fn new() -> Self {
        Self {
            object_space: markedspace::MarkedSpace::new(),
        }
    }
}
