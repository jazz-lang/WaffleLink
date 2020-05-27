#[repr(C)]
pub struct MiHeap {
    _field: [u8; 0],
}

#[link(name = "mimalloc")]
extern "C" {
    fn mi_heap_new() -> *mut MiHeap;
    fn mi_heap_destroy(ptr: *mut MiHeap);
    fn mi_heap_collect(ptr: *mut MiHeap, force: bool);
    fn mi_heap_malloc(heap: *mut MiHeap, size: usize) -> *mut u8;
    fn mi_heap_mallocn(heap: *mut MiHeap, count: usize, size: usize) -> *mut u8;
    fn mi_free(ptr: *mut u8);
}

pub struct Heap {
    heap: &'static mut MiHeap,
}

impl Heap {
    #[inline]
    pub fn new() -> Self {
        Self {
            heap: unsafe { &mut *mi_heap_new() },
        }
    }

    #[inline]
    pub fn alloc(&mut self, size: usize) -> *mut u8 {
        unsafe { mi_heap_malloc(self.heap, size) }
    }
    #[inline]
    pub fn allocn(&mut self, count: usize, size: usize) -> *mut u8 {
        unsafe { mi_heap_mallocn(self.heap, count, size) }
    }
}

impl Drop for Heap {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            mi_heap_collect(self.heap, true);
        }
        unsafe { mi_heap_destroy(self.heap) }
    }
}

cfg_if::cfg_if!(
    if #[cfg(feature="single-threaded")]
    {

        #[inline(always)]
        pub fn gc_safepoint() {
        }

        pub fn stop_the_world(f: impl FnMut()) {
        }
    } else if #[cfg(feature="multi-threaded")] {
        use std::sync::atomic::{AtomicBool,Ordering};
        static GC_IS_RUNNING: AtomicBool = AtomicBool::new(false);
        /// GC safepoint. Used to suspend thread if GC cycle is needed.
        #[inline]
        pub fn gc_safepoint() {
            let mut attempt = 0;
            while GC_IS_RUNNING.load(Ordering::Acquire) {
                if attempt > 3 {
                    std::thread::sleep(std::time::Duration::from_micros(2000));
                } else {
                    attempt += 1;
                    std::thread::yield_now();
                }
            }
        }
    }
);

pub struct GcHeader<T: Collectable + ?Sized> {
    pub(crate) mark_bit: u8,
    pub(crate) lock: super::lock::MLock,
    pub(crate) next: *mut GcHeader<dyn Collectable>,
    pub(crate) value: T,
}
pub struct Handle<T: Collectable + ?Sized> {
    inner: std::ptr::NonNull<GcHeader<T>>,
}

impl<T: Collectable + ?Sized> Collectable for Handle<T> {
    fn walk_references(&self, trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        trace(self as *const Self as *const Handle<dyn Collectable>);
    }
}
pub trait Collectable {
    fn walk_references(&self, trace: &mut dyn FnMut(*const Handle<dyn Collectable>));
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
    mi_heap: Heap,
    list: *mut GcHeader<dyn Collectable>,
    grey: *mut GcHeader<dyn Collectable>,
}

const GRAY: u8 = 0;
const WHITE_A: u8 = 1;
const WHITE_B: u8 = 1 << 1;
const BLACK: u8 = 1 << 2;
const GC_WHITES: u8 = WHITE_A | WHITE_B;
const COLOR_MASK: u8 = 7;

const fn is_black(o: &GcHeader<dyn Collectable>) -> bool {
    (o.mark_bit & BLACK) != 0
}

const fn is_white(o: &GcHeader<dyn Collectable>) -> bool {
    (o.mark_bit & GC_WHITES) != 0
}

const fn is_gray(o: &GcHeader<dyn Collectable>) -> bool {
    o.mark_bit == GRAY
}

impl WaffleHeap {
    unsafe fn sweep_all(&mut self) {
        while !self.list.is_null() {
            let val = &mut *self.list;
            let next = val.next;
            if is_black(val) {
                // TODO
            } else {
                std::ptr::drop_in_place(self.list);
                mi_free(self.list as *mut u8);
            }
            self.list = next;
        }
    }

    unsafe fn mark(&mut self) {
        while !self.grey.is_null() {
            let hdr = &mut *self.grey;
            let next = hdr.next;
            hdr.value.walk_references(|handle_ptr| {
                let handle = &mut *handle_ptr;
                let inner_ptr = handle.inner;
                let inner = &mut *inner_ptr;
                if inner.mark_bit {
                    return;
                } else {
                }
            });
        }
    }
}
