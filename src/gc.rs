//! # CakeGC
//!  Generational mark&sweep garbage collector.
//!
//!
//! # Features
//! - Non moving GC
//! - Generational
//! - Easy to use (no write barriers,safepoints etc)
//!
//!
//! # How it works?
//! CakeGC is non-moving so many users might ask how does it work like generational?
//! Answer is pretty easy: we maintain two singly linked lists, one for young generation heap
//! and second one is for old generation and when sweeping we promote young to old.
//! Also there is no write barriers needed for this GC to work because it is not incremental
//! and will visit all references inside GC value, this might be slow but it works.

pub mod cmarking;
pub mod object;
pub mod pagealloc;
#[cfg(feature = "pmarking")]
pub mod pmarking;
use object::*;
use std::collections::VecDeque;
/// GC didn't seen this object.
pub const GC_WHITE: u8 = 0;
/// Object fields was visited
pub const GC_BLACK: u8 = 2;
/// Object is in graylist
pub const GC_GRAY: u8 = 1;
/// Old gen object
pub const GC_BLUE: u8 = 3;
pub const GC_NEW: u8 = 0;
pub const GC_OLD: u8 = 1;
pub const GC_NONE: u8 = 3;
pub const GC_OLD_REMEMBERED: u8 = 2;

pub const GC_VERBOSE_LOG: bool = !true;
pub const GC_LOG: bool = !true;

pub struct TGC {
    eden: *mut GcBox<()>,
    eden_size: usize,
    eden_allowed_size: usize,
    old: *mut GcBox<()>,
    old_size: usize,
    old_allowed_size: usize,
    remembered: Vec<*mut GcBox<()>>,
    gc_ty: u8,
    graystack: VecDeque<*mut GcBox<()>>,
    blacklist: Vec<*mut GcBox<()>>,
    mi_heap: *mut libmimalloc_sys::mi_heap_t,
    stack_begin: *const usize,
    stack_end: *const usize,

    #[cfg(feature = "pmarking")]
    pool: threadpool::ThreadPool,
    #[cfg(feature = "pmarking")]
    num_cpus: usize,
    pmark: bool,
    handles: Vec<*mut LocalScopeInner>,
}

impl TGC {
    pub fn write_barrier<T: GcObject, U: GcObject>(&mut self, object: Handle<T>, field: Handle<U>) {
        unsafe {
            let object = &mut *object.gc_ptr();
            let field = &mut *field.gc_ptr();
            if object.header.next_tag() == GC_OLD && object.header.next_tag() == GC_NEW {
                object.header.set_next_tag(GC_OLD_REMEMBERED);
                self.remembered.push(object);
            }
        }
    }

    pub fn new(begin: *const usize, n_cpus: Option<usize>, pmark: bool) -> Self {
        Self {
            remembered: vec![],
            #[cfg(feature = "pmarking")]
            num_cpus: n_cpus.unwrap_or(num_cpus::get() / 2),
            #[cfg(feature = "pmarking")]
            pool: threadpool::Builder::new()
                .num_threads(n_cpus.unwrap_or(num_cpus::get() / 2))
                .thread_stack_size(64 * 1024)
                .build(),
            eden_allowed_size: 8 * 1024,
            eden_size: 0,
            old: 0 as *mut _,
            old_allowed_size: 16 * 1024,
            old_size: 0,
            eden: 0 as *mut _,
            pmark,
            handles: vec![],
            gc_ty: GC_NONE,

            graystack: Default::default(),
            blacklist: Vec::new(),
            mi_heap: unsafe { libmimalloc_sys::mi_heap_new() },
            stack_begin: begin,
            stack_end: begin,
        }
    }

    fn mark_object(&mut self, object: *mut GcBox<()>) {
        //panic!();
        unsafe {
            let obj = &mut *object;
            if obj.header.white_to_gray() {
                if GC_VERBOSE_LOG && GC_LOG {
                    println!("---mark {:p}", object);
                }
                self.graystack.push_back(object);
            }
        }
    }
    pub fn new_local_scope(&mut self) -> LocalScope {
        let inner = Box::into_raw(Box::new(LocalScopeInner {
            locals: Vec::new(),
            gc: self as *mut _,
            dead: false,
        }));

        self.handles.push(inner);
        LocalScope { inner }
    }
    pub fn force_major_gc(&mut self) {
        let stack = 0usize;
        self.collect_garbage(&stack);
        self.gc_ty = GC_OLD;
        self.collect_garbage(&stack);
    }

    fn sweep(&mut self, eden: bool) {
        let mut count = 0;
        let mut freed = 0;

        if eden {
            if GC_LOG {
                println!("--begin eden sweep");
            }
            let mut object = self.eden;
            self.eden_size = 0;
            while object.is_null() == false {
                unsafe {
                    let obj = &mut *object;
                    let next = obj.header.next();
                    let size = obj.trait_object().size() + core::mem::size_of::<Header>();

                    if obj.header.tag() == GC_BLACK {
                        obj.header.set_tag(GC_WHITE);
                        obj.header.set_next(self.old as _);
                        obj.header.set_next_tag(GC_OLD);
                        self.old = obj as *mut _;
                        if GC_VERBOSE_LOG {
                            println!("---promote {:p}", obj);
                        }
                        self.old_size += size;
                    } else {
                        #[cfg(debug_assertions)]
                        {
                            assert!(obj.header.tag() != GC_GRAY);
                        }
                        let unreached = object;

                        freed += size;

                        if GC_LOG && GC_VERBOSE_LOG {
                            println!("---sweep eden {:p} size={}", unreached, size);
                        }
                        obj.trait_object().finalize();
                        core::ptr::drop_in_place(obj.trait_object());
                        libmimalloc_sys::mi_free(unreached.cast());
                        count += 1;
                    }
                    object = next.cast();
                }
            }
            self.eden = 0 as *mut _;
        } else {
            if GC_LOG {
                println!("--begin old sweep");
            }
            unsafe {
                let mut previous: *mut GcBox<()> = core::ptr::null_mut();
                let mut object = self.old;
                self.old = 0 as *mut _;
                let mut new_old = 0 as *mut GcBox<()>;
                while !object.is_null() {
                    let obj = &mut *object;
                    let next = obj.header.next();
                    let size = obj.trait_object().size() + core::mem::size_of::<Header>();
                    if obj.header.tag() != GC_WHITE {
                        obj.header.set_tag(GC_WHITE);
                        obj.header.set_next(new_old as _);
                        obj.header.set_next_tag(GC_OLD);

                        new_old = object;
                    } else {
                        #[cfg(debug_assertions)]
                        {
                            assert!(obj.header.tag() != GC_GRAY);
                        }
                        let unreached = object;

                        freed += size;

                        if GC_LOG && GC_VERBOSE_LOG {
                            println!(
                                "---sweep old {:p} size={}, color={}",
                                unreached,
                                size,
                                obj.header.tag()
                            );
                        }
                        core::ptr::drop_in_place(obj.trait_object());
                        libmimalloc_sys::mi_free(unreached.cast());
                        count += 1;
                    }
                    object = next as _;
                }
                self.old = new_old;
            }
        }

        if GC_LOG {
            println!("--sweeped {} object(s) ({} bytes)", count, freed);
        }
    }
    fn visit_roots(&mut self, visit: &mut dyn FnMut(*mut GcBox<()>)) {
        let start = std::time::Instant::now();
        self.handles.retain(|handle| unsafe {
            let mut handle = &mut **handle;

            if handle.dead {
                let _ = Box::from_raw(handle);
                false
            } else {
                handle.locals.retain(|local| {
                    if local.is_null() {
                        false
                    } else {
                        visit(*local);
                        true
                    }
                });
                true
            }
        });
    }
    fn mark_from_roots(&mut self) {
        let start = std::time::Instant::now();
        #[cfg(feature = "pmarking")]
        {
            if self.pmark {
                if GC_LOG {
                    println!("--start parallel marking",);
                }
                let mut roots = vec![];
                self.visit_roots(&mut |object| unsafe {
                    let obj = &mut *object;
                    if obj.header.tag() == GC_WHITE {
                        if GC_VERBOSE_LOG {
                            println!("---root {:p}", obj);
                        }
                        obj.header.set_tag(GC_GRAY);
                        roots.push(Address::from_ptr(object));
                    }
                });
                /*self.roots.walk(&mut |root| unsafe {
                    if GC_VERBOSE_LOG {
                        println!("---root {:p}", (&*root).obj);
                    }
                    let obj = (&*root).obj;
                    if (&*obj).header.tag() == GC_WHITE {
                        (&mut *obj).header.set_tag(GC_GRAY);
                        roots.push(Address::from_ptr((&*root).obj));
                    }
                });*/

                let mut count = 0;
                let marking_start = std::time::Instant::now();
                let _ = pmarking::start(&roots, self.num_cpus, &mut self.pool, self.gc_ty);
                let end = marking_start.elapsed();
                //println!("pmark in {}ns", end.as_nanos());

                if GC_LOG {
                    println!("--parallel marking finished");
                }
            } else {
                if GC_LOG {
                    println!("--mark from roots");
                }

                let this = unsafe { &mut *(self as *mut Self) };
                /*this.roots.walk(&mut |root| unsafe {
                                    if GC_VERBOSE_LOG {
                                        println!("---root {:p}", (&*root).obj);
                                    }
                                    self.mark_object((&*root).obj);
                                });
                */
                this.visit_roots(&mut |root| {
                    self.mark_object(root);
                });
                if GC_LOG {
                    println!("--process gray stack");
                }
                self.process_gray();
                while let Some(object) = self.blacklist.pop() {
                    unsafe {
                        (&mut *object).header.set_tag(GC_WHITE);
                    }
                }
            }
        }
        #[cfg(not(feature = "pmarking"))]
        {
            if self.pmark {
                panic!("WaffleLink compiled without support for parallel marking!");
            }
            if GC_LOG {
                println!("--mark from roots");
            }

            let this = unsafe { &mut *(self as *mut Self) };
            self.visit_roots(&mut |root| {
                this.mark_object(root);
            });
            if GC_LOG {
                println!("--process gray stack");
            }
            self.process_gray();
            while let Some(object) = self.blacklist.pop() {
                unsafe {
                    (&mut *object).header.set_tag(GC_WHITE);
                }
            }
        }
    }
    #[inline(never)]
    pub fn collect_garbage(&mut self, stack_end: *const usize) {
        if self.gc_ty == GC_NONE {
            self.gc_ty = GC_NEW;
        }
        self.stack_end = stack_end;
        if GC_LOG {
            println!(
                "--GC begin, heap size {}(threshold {})",
                self.eden_size, self.eden_allowed_size
            );
        }
        self.mark_from_roots();
        let before = self.old_size;
        self.sweep(self.gc_ty == GC_NEW);
        let prev = self.gc_ty;
        if self.old_allowed_size <= self.old_size && self.gc_ty == GC_NEW {
            if GC_LOG {
                println!("--starting old space GC");
            }
            self.gc_ty = GC_OLD;
            self.collect_garbage(stack_end);
        }
        if prev != GC_NEW {
            let freed = before - self.old_size;
            if before as f64 * 0.5 <= freed as f64 {
                unsafe {
                    if GC_LOG {
                        println!("--mi heap collect {} <= {}", before as f64 * 0.65, freed);
                    }
                    libmimalloc_sys::mi_heap_collect(self.mi_heap, true);
                }
            }
        }

        if self.eden_size >= self.eden_allowed_size {
            self.eden_allowed_size = (self.eden_size as f64 / 0.75) as usize;
        }
        if self.old_size >= self.old_allowed_size {
            self.old_allowed_size = (self.old_size as f64 / 0.5) as usize;
        }
        if GC_LOG {
            println!(
                "--GC end, eden threshold {},old threshold {}\n",
                self.eden_allowed_size, self.old_allowed_size
            );
        }
        self.gc_ty = GC_NONE;
    }
    pub fn allocate_no_root<T: GcObject>(&mut self, value: T) -> Handle<T> {
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
                    is_old: false,
                    cell_state: GC_WHITE,
                    //cell_state: std::sync::atomic::AtomicU8::new(GC_WHITE),
                    vtable: core::mem::transmute::<_, TraitObject>(&value as &dyn GcObject).vtable,
                    next: 0 as *mut _,
                },
                value,
            });
            (&mut *mem).header.set_tag(0);
            (&mut *mem).header.set_next(self.eden.cast());
            (&mut *mem).header.set_next_tag(0);
            if GC_VERBOSE_LOG {
                println!("--allocate {:p}", mem);
            }
            self.eden = mem.cast();
            Handle {
                ptr: core::ptr::NonNull::new(mem).unwrap(),
            }
        }
    }
    pub fn allocate<T: GcObject>(&mut self, value: T) -> Handle<T> {
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
                    is_old: false,
                    cell_state: GC_WHITE,
                    //cell_state: std::sync::atomic::AtomicU8::new(GC_WHITE),
                    vtable: core::mem::transmute::<_, TraitObject>(&value as &dyn GcObject).vtable,
                    next: 0 as *mut _,
                },
                value,
            });
            (&mut *mem).header.set_tag(0);
            (&mut *mem).header.set_next(self.eden.cast());
            (&mut *mem).header.set_next_tag(0);
            if GC_VERBOSE_LOG {
                println!("--allocate {:p}", mem);
            }
            self.eden = mem.cast();
            Handle {
                ptr: core::ptr::NonNull::new(mem).unwrap(),
            }
        }
    }

    fn visit_value(&mut self, obj: &mut GcBox<()>) {
        obj.trait_object().visit_references(&mut |object| {
            self.mark_object(object as *mut _);
        })
    }
    fn process_gray(&mut self) {
        while let Some(item) = self.graystack.pop_front() {
            unsafe {
                let obj = &mut *item;
                if obj.header.next_tag() != self.gc_ty {
                    if obj.header.to_blue() {
                        if GC_VERBOSE_LOG {
                            println!("---blacklist {:p}", obj);
                        }
                        self.visit_value(obj);
                        self.blacklist.push(item);
                    }
                } else if obj.header.gray_to_black() {
                    if GC_VERBOSE_LOG && GC_LOG {
                        println!("---mark {:p}", obj);
                    }

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
