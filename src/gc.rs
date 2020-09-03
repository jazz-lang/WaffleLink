//! # CakeGC
//!  Generational mark&sweep garbage collector.
//!
//!
//! # Features
//! - Non moving GC
//! - Generational
//! - Write barriers protected for fast GC cycles.
//!
//!
//! # How it works?
//! CakeGC is non-moving so many users might ask how does it work like generational?
//! Answer is pretty easy: we maintain two singly linked lists, one for young generation heap
//! and second one is for old generation and when sweeping we promote young to old.

use crate::timer::Timer;
use core::sync::atomic::Ordering;
pub type GCObjectRef = *mut GcBox<()>;
/// Cast reference to atomic type
#[macro_export]
macro_rules! as_atomic {
    ($val: expr,$t: ty) => {
        &*($val as *const _ as *const $t)
    };

    (ref $val: expr,$t: ty) => {
        unsafe { &*(&$val as *const _ as *const $t) }
    };
}
pub trait GcObject {
    /// Returns size of current object. This is usually just `size_of_val(self)` but in case
    /// when you need dynamically sized type e.g array then this should return something like
    /// `size_of(Base) + length * size_of(Value)`
    fn size(&self) -> usize {
        core::mem::size_of_val(self)
    }

    fn visit_references(&self, trace: &mut dyn FnMut(*const GcBox<()>)) {}
}

pub struct Header {
    pub vtable: TaggedPointer<()>,
    pub next: TaggedPointer<GcBox<()>>,
    pub flags: u8,
}

impl Header {
    pub fn is_old(&self) -> bool {
        self.vtable.bit_is_set(0)
    }
    pub fn set_old(&mut self) {
        self.vtable.set_bit(0);
    }

    pub fn set_young(&mut self) {
        self.vtable.clear(0);
    }
    pub fn is_young(&self) -> bool {
        !self.is_old()
    }

    pub fn is_marked(&self) -> bool {
        self.vtable.bit_is_set(2)
        //(self.flags & 0x80) != 0
    }

    pub fn test_and_mark(&mut self) -> bool {
        if self.is_marked() {
            false
        } else {
            self.vtable.set_bit(2);
            //self.flags |= 0x80;
            true
        }
    }

    pub fn mark(&mut self) {
        self.vtable.set_bit(2);
        //self.flags |= 0x80;
    }

    pub fn unmark(&mut self) {
        self.vtable.clear(2);
        //self.flags ^= 0x80;
    }
    pub fn test_soft_mark(&mut self) -> bool {
        /*if (self.flags & 0x40) != 0 {
            false
        } else {
            self.flags |= 0x40;
            true
        }*/
        if self.is_soft_marked() {
            false
        } else {
            self.vtable.set_bit(1);
            true
        }
    }
    pub fn is_soft_marked(&self) -> bool {
        //(self.flags & 0x40) != 0
        self.vtable.bit_is_set(1)
    }
    pub fn clear_soft_mark(&mut self) {
        if self.is_soft_marked() {
            self.vtable.clear(1);
            //self.flags ^= 0x40;
        }
    }

    /*pub fn is_remembered(&self) -> bool {
        self.next.bit_is_set(0)
    }

    pub fn set_remembered(&mut self)  {
        self.next.set_bit(0)
    }

    pub fn clear_remembered(&mut self) {
        self.next.clear(0)
    }*/
}

#[repr(C)]
pub struct GcBox<T: GcObject> {
    header: Header,
    value: T,
}

#[repr(C)]
struct TraitObject {
    data: *mut (),
    vtable: *mut (),
}

impl<T: GcObject> GcBox<T> {
    pub fn trait_object(&self) -> &mut dyn GcObject {
        unsafe {
            core::mem::transmute(TraitObject {
                data: &self.value as *const _ as *mut _,
                vtable: self.header.vtable.untagged(),
            })
        }
    }
}

impl<T: GcObject> GcObject for Root<T> {
    fn visit_references(&self, trace: &mut dyn FnMut(*const GcBox<()>)) {
        self.to_heap().visit_references(trace);
    }
}

impl GcObject for () {}
use std::collections::VecDeque;
pub struct Heap {
    generational: bool,
    /// Singly linked list of young objects
    young: *mut GcBox<()>,
    /// Singly linked list of old objects
    old: *mut GcBox<()>,
    /// After major_threshold we start major GC
    major_threshold: usize,
    /// Currently allocated major objects
    major_size: usize,
    /// Currently allocated young objects
    young_size: usize,
    /// After young_threshold we start minor GC
    young_threshold: usize,
    /// Remembred set of old objects, used in generational mode.
    remembered: Vec<*mut GcBox<()>>,
    blacklist: std::vec::Vec<*mut GcBox<()>>,
    graylist: VecDeque<*mut GcBox<()>>,
    gc_ty: GcType,
    roots: RootList,
    stats: CollectionStats,
    verbose: bool,
    cycle_stats: bool,
    cur_stats: CycleStats,
}
pub struct RootInner {
    rc: u32,
    pub obj: *mut GcBox<()>,
}

/// Rooted value. All GC allocated values wrapped in `Root<T>` will be scanned by GC for
/// references. You can use this type like regular `Rc` but please try to minimize it's usage
/// because GC allocates some heap memory for managing roots.
pub struct Root<T: GcObject> {
    inner: *mut RootInner,
    _marker: core::marker::PhantomData<T>,
}

pub struct RootList {
    roots: Vec<*mut RootInner>,
}
use std::{boxed::Box, vec::Vec};
impl RootList {
    pub fn new() -> Self {
        Self {
            roots: Vec::with_capacity(4),
        }
    }
    pub fn root<T: GcObject>(&mut self, o: *mut GcBox<T>) -> Root<T> {
        let root = Box::into_raw(Box::new(RootInner {
            rc: 1,
            obj: o.cast(),
        }));
        self.roots.push(root);
        Root {
            inner: root,
            _marker: Default::default(),
        }
    }
    pub fn unroot<T: GcObject>(&mut self, r: Root<T>) {
        drop(r)
    }

    pub(crate) fn walk(&mut self, walk: &mut dyn FnMut(*const RootInner)) {
        /*let mut cur = self.roots;
        while cur.is_non_null() {
            walk(cur);
            cur = cur.next;
        }*/

        self.roots.retain(|x| unsafe {
            if (&**x).rc == 0 {
                let _ = std::boxed::Box::from_raw(*x);
                false
            } else {
                walk(*x);
                true
            }
        });
    }
}

impl Heap {
    pub fn collect_garbage_force(&mut self, ty: GcType) {
        self.gc_ty = ty;
        self.collect_garbage();
    }
    pub fn write_barrier<T: GcObject, U: GcObject>(
        &mut self,
        holder: Handle<T>,
        new_value: Handle<U>,
    ) {
        unsafe {
            let raw = &mut *holder.gc_ptr();
            if raw.header.is_old() && raw.header.test_soft_mark() {
                let raw2 = &*new_value.gc_ptr();
                
                if raw2.header.is_young() {
                    self.remembered.push(holder.gc_ptr() as *mut _);
                }
            }
        }
    }

    pub fn new(verbose: bool, cycle_stats: bool) -> Self {
        Self {
            cycle_stats,
            generational: true,
            young: core::ptr::null_mut(),
            old: core::ptr::null_mut(),
            graylist: Default::default(),
            blacklist: Default::default(),
            major_size: 0,
            cur_stats: Default::default(),
            stats: CollectionStats::new(),
            major_threshold: 32 * 1024,
            young_size: 0,
            young_threshold: 8 * 1024,
            gc_ty: GcType::None,
            remembered: Default::default(),
            roots: RootList::new(),
            verbose,
        }
    }
    pub fn should_collect(&self) -> bool {
        self.gc_ty != GcType::None
            || self.young_size >= self.young_threshold
            || self.major_threshold >= self.major_threshold
    }
    pub fn root<T: GcObject>(&mut self, handle: Handle<T>) -> Root<T> {
        self.roots.root(handle.gc_ptr() as *mut _)
    }
    pub fn allocate<T: GcObject>(&mut self, val: T) -> Root<T> {
        unsafe {
            let alloc_size = val.size() + core::mem::size_of::<Header>();
            if alloc_size + self.young_size >= self.young_threshold {
                self.collect_garbage();
            }
            let alignment = 16;
            let layout = std::alloc::Layout::from_size_align_unchecked(alloc_size, alignment)
                .align_to(8)
                .unwrap();
            let mem = std::alloc::alloc_zeroed(layout).cast::<GcBox<T>>();
            mem.write(GcBox {
                header: Header {
                    next: TaggedPointer::null(),
                    vtable: TaggedPointer::new({
                        core::mem::transmute::<_, TraitObject>(&val as &dyn GcObject).vtable
                    }),
                    flags: 0,
                },
                value: val,
            });

            self.push_young(mem.cast());
            let root = self.roots.root(mem);
            root
        }
    }

    pub fn collect_garbage(&mut self) {
        let mut timer = Timer::new(self.verbose);
        let start = time::Instant::now();
        if self.gc_ty == GcType::None {
            self.gc_ty = GcType::Minor;
        }
        if self.verbose {
            //println!("{:?} GC cycle started", self.gc_ty);
        }
        let this = self as *const Self as *mut Self;
        let this = unsafe { &mut *this };
        this.roots.walk(&mut |root| unsafe {
            let root = &*root;
            self.graylist.push_back(root.obj);
        });
        self.process_remembered();
        self.process_gray();
        if self.gc_ty == GcType::Minor {
            let mut head = self.young;
            self.young = core::ptr::null_mut();
            while !head.is_null() {
                let val = unsafe { &mut *head };
                let next = val.header.next.untagged();

                if val.header.is_marked() {
                    val.header.unmark();
                    self.cur_stats.promoted += 1;
                    self.promote(head);
                } else {
                    let size = val.trait_object().size() + core::mem::size_of::<Header>();
                    self.young_size -= size;
                    self.cur_stats.sweeped += 1;
                    self.cur_stats.freed += size;
                    unsafe {
                        core::ptr::drop_in_place(val.trait_object());
                    }
                }
                head = next;
            }
        } else {
            let mut head = self.old;
            self.old = core::ptr::null_mut();
            let mut last = core::ptr::null_mut();
            while !head.is_null() {
                let val = unsafe { &mut *head };
                let next = val.header.next.untagged();
                debug_assert!(!val.header.is_soft_marked());
                if val.header.is_marked() {
                    
                    val.header.unmark();
                    last = head;
                } else {
                    unsafe {
                        (&mut*last).header.next = TaggedPointer::new(next);
                    }
                    let size = val.trait_object().size() + core::mem::size_of::<Header>();
                    self.major_size -= size;
                    self.cur_stats.sweeped += 1;
                    self.cur_stats.freed += size;
                    unsafe {
                        core::ptr::drop_in_place(val.trait_object());
                    }
                }
                
                head = next;
            }
        }

        for item in self.blacklist.drain(..) {
            unsafe {
                (&mut *item).header.clear_soft_mark();
            }
        }

        let cur_gc = self.gc_ty;
        let cstats = self.cur_stats.clone();
        if cur_gc == GcType::Minor && self.major_size >= self.major_threshold {
            self.gc_ty = GcType::Major;
            self.collect_garbage();
        }
        if cur_gc == GcType::Minor && self.young_size >= self.young_threshold {
            self.young_threshold = (self.young_size as f64 * 0.75) as usize;
        }
        if cur_gc == GcType::Major && self.major_size >= self.major_threshold {
            self.major_threshold = (self.major_size as f64 * 0.55) as usize;
        }
        self.gc_ty = GcType::None;
        self.blacklist.shrink_to_fit();
        let end = start.elapsed();
        if self.verbose {
            let duration = timer.stop();
            self.stats.add(duration);
        }
        if self.cycle_stats {
            eprintln!(
                "{:?} cycle finished in {}ms({}ns): \n Sweeped={}\n Freed={} \n Remembered={} \n Promoted={}",
                
                cur_gc,end.whole_milliseconds(),
                end.whole_nanoseconds(),
                cstats.sweeped,
                formatted_size(cstats.freed),
                cstats.remembered,
                cstats.promoted
            );
        }
        self.cur_stats = Default::default();
        assert!(self.blacklist.is_empty());
        assert!(self.graylist.is_empty());
        assert!(self.remembered.is_empty());
    }
    pub fn dump_summary(&self, runtime: f32) {
        let stats = &self.stats;
        let (mutator, gc) = stats.percentage(runtime);

        println!("GC stats: total={:.1}", runtime);
        println!("GC stats: mutator={:.1}", stats.mutator(runtime));
        println!("GC stats: collection={:.1}", stats.pause());

        println!("");
        println!("GC stats: collection-count={}", stats.collections());
        println!("GC stats: collection-pauses={}", stats.pauses());

        println!(
            "GC summary: {:.1}ms collection ({}), {:.1}ms mutator, {:.1}ms total ({}% mutator, {}% GC)",
            stats.pause(),
            stats.collections(),
            stats.mutator(runtime),
            runtime,
            mutator,
            gc,
        );
    }

    pub fn in_current_space(&self, obj: GCObjectRef) -> bool {
        if self.gc_ty == GcType::Major {
            unsafe { (&*obj).header.is_old() }
        } else {
            unsafe { (&*obj).header.is_young() }
        }
    }

    fn push_young(&mut self, obj: GCObjectRef) {
        let o = unsafe { &mut *obj };
        self.young_size += o.trait_object().size() + core::mem::size_of::<Header>();
        o.header.next = TaggedPointer::new(self.young);
        o.header.set_young();
        self.young = obj;
    }
    fn push_old(&mut self, obj: GCObjectRef) {
        let o = unsafe { &mut *obj };
        self.major_size += o.trait_object().size() + core::mem::size_of::<Header>();

        o.header.next = TaggedPointer::new(self.old);
        o.header.set_old();
        self.old = obj;
    }
    fn promote(&mut self, obj: GCObjectRef) -> bool {
        let o = unsafe { &mut *obj };
        if o.header.is_young() {
            self.push_old(obj);
            true
        } else {
            false
        }
    }

    fn visit_value(&mut self, val: &mut GcBox<()>) {
        let object = val.trait_object();

        object.visit_references(&mut |item| {
            self.graylist.push_back(item as *mut _);
        });
    }
    fn process_remembered(&mut self) {
        if self.gc_ty == GcType::Minor {
            // in young space collection push all references from old to young to graylist.
            while let Some(old) = self.remembered.pop() {
                unsafe {
                    let old_ref = &mut *old;
                    if old_ref.header.is_soft_marked() {
                       
                        self.cur_stats.remembered += 1;
                        old_ref.trait_object().visit_references(&mut |item| {
                            let item_ref = &mut *(item as *mut GcBox<()>);
                            if item_ref.header.is_old() {
                                return;
                            } else {
                                self.graylist.push_back(item as *mut _);
                            }
                        });
                        self.blacklist.push(old);
                    } else {
                        unreachable!();
                    }
                }
            }
        } else {
            // in major collection empty remembered set.
            self.remembered.drain(..).for_each(|x| {
                unsafe {
                    (&mut*x).header.clear_soft_mark();
                }
            })
        }
    }
    fn process_gray(&mut self) {
        while let Some(value) = self.graylist.pop_front() {
            let val = unsafe { &mut *value };
            if !self.in_current_space(value) {
                if val.header.test_soft_mark() {
                    self.blacklist.push(value);
                    if self.gc_ty == GcType::Major {
                        self.visit_value(val);
                    }
                }
                continue;
            }

            if val.header.test_and_mark() {
                self.visit_value(val);
            }
        }
    }
}

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub enum GcType {
    Minor,
    Major,
    None,
}

use core::hash::{Hash, Hasher};
use core::ptr;
use core::sync::atomic::AtomicPtr;

/// The mask to use for untagging a pointer.
const UNTAG_MASK: usize = (!0x7) as usize;

/// Returns true if the pointer has the given bit set to 1.
pub fn bit_is_set<T>(pointer: *mut T, bit: usize) -> bool {
    let shifted = 1 << bit;

    (pointer as usize & shifted) == shifted
}

/// Returns the pointer with the given bit set.
pub fn with_bit<T>(pointer: *mut T, bit: usize) -> *mut T {
    (pointer as usize | 1 << bit) as _
}

/// Returns the given pointer without any tags set.
pub fn untagged<T>(pointer: *mut T) -> *mut T {
    (pointer as usize & UNTAG_MASK) as _
}

/// Structure wrapping a raw, tagged pointer.
#[derive(Debug)]
pub struct TaggedPointer<T> {
    pub raw: *mut T,
}

impl<T> TaggedPointer<T> {
    /// Returns a new TaggedPointer without setting any bits.
    pub fn new(raw: *mut T) -> TaggedPointer<T> {
        TaggedPointer { raw }
    }

    /// Returns a new TaggedPointer with the given bit set.
    pub fn with_bit(raw: *mut T, bit: usize) -> TaggedPointer<T> {
        let mut pointer = Self::new(raw);

        pointer.set_bit(bit);

        pointer
    }
    pub fn clear(&mut self, bit: usize) {
        let shifted = 1 << bit;
        self.raw = (self.raw as usize & !shifted) as *mut T;
    }
    /// Returns a null pointer.
    pub fn null() -> TaggedPointer<T> {
        TaggedPointer {
            raw: ptr::null::<T>() as *mut T,
        }
    }

    /// Returns the wrapped pointer without any tags.
    pub fn untagged(self) -> *mut T {
        self::untagged(self.raw)
    }

    /// Returns a new TaggedPointer using the current pointer but without any
    /// tags.
    pub fn without_tags(self) -> Self {
        Self::new(self.untagged())
    }

    /// Returns true if the given bit is set.
    pub fn bit_is_set(self, bit: usize) -> bool {
        self::bit_is_set(self.raw, bit)
    }

    /// Sets the given bit.
    pub fn set_bit(&mut self, bit: usize) {
        self.raw = with_bit(self.raw, bit);
    }

    /// Returns true if the current pointer is a null pointer.
    pub fn is_null(self) -> bool {
        self.untagged().is_null()
    }

    /// Returns an immutable to the pointer's value.
    pub fn as_ref<'a>(self) -> Option<&'a T> {
        unsafe { self.untagged().as_ref() }
    }

    /// Returns a mutable reference to the pointer's value.
    pub fn as_mut<'a>(self) -> Option<&'a mut T> {
        unsafe { self.untagged().as_mut() }
    }

    /// Atomically swaps the internal pointer with another one.
    ///
    /// This boolean returns true if the pointer was swapped, false otherwise.
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]
    pub fn compare_and_swap(&self, current: *mut T, other: *mut T) -> bool {
        self.as_atomic()
            .compare_and_swap(current, other, Ordering::AcqRel)
            == current
    }

    /// Atomically replaces the current pointer with the given one.
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]
    pub fn atomic_store(&self, other: *mut T) {
        self.as_atomic().store(other, Ordering::Release);
    }

    /// Atomically loads the pointer.
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]
    pub fn atomic_load(&self) -> *mut T {
        self.as_atomic().load(Ordering::Acquire)
    }

    /// Checks if a bit is set using an atomic load.
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]
    pub fn atomic_bit_is_set(&self, bit: usize) -> bool {
        Self::new(self.atomic_load()).bit_is_set(bit)
    }

    fn as_atomic(&self) -> &AtomicPtr<T> {
        unsafe { &*(self as *const TaggedPointer<T> as *const AtomicPtr<T>) }
    }
}

impl<T> PartialEq for TaggedPointer<T> {
    fn eq(&self, other: &TaggedPointer<T>) -> bool {
        self.raw == other.raw
    }
}

impl<T> Eq for TaggedPointer<T> {}

// These traits are implemented manually as "derive" doesn't handle the generic
// "T" argument very well.
impl<T> Clone for TaggedPointer<T> {
    fn clone(&self) -> TaggedPointer<T> {
        TaggedPointer::new(self.raw)
    }
}

impl<T> Copy for TaggedPointer<T> {}

impl<T> Hash for TaggedPointer<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<T: Hash + GcObject> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: PartialEq + GcObject> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        (**self).eq(&**other)
    }
}

impl<T: Eq + GcObject> Eq for Handle<T> {}
impl<T: PartialOrd + GcObject> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }
}
impl<T: Ord + GcObject> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (**self).cmp(&**other)
    }
}

use std::fmt::{self, Formatter};

impl<T: fmt::Display + GcObject> fmt::Display for Handle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", **self)
    }
}
impl<T: fmt::Debug + GcObject> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", **self)
    }
}
pub struct Handle<T: GcObject> {
    ptr: core::ptr::NonNull<GcBox<T>>,
}

impl<T: GcObject> Handle<T> {
    pub fn gc_ptr(&self) -> *mut GcBox<()> {
        self.ptr.cast::<_>().as_ptr()
    }
    pub fn ptr_eq(this: Self, other: Self) -> bool {
        this.ptr == other.ptr
    }
    pub unsafe fn from_raw<U>(x: *const U) -> Self {
        Self {
            ptr: core::ptr::NonNull::new((x as *mut U).cast()).unwrap(),
        }
    }
}
impl<T: GcObject> Root<T> {
    pub fn to_heap(&self) -> Handle<T> {
        Handle {
            ptr: unsafe {
                core::ptr::NonNull::new_unchecked((&*self.inner).obj.cast::<GcBox<T>>() as *mut _)
            },
        }
    }
}
impl<T: GcObject> core::ops::Deref for Handle<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &(&*self.ptr.as_ptr()).value }
    }
}

impl<T: GcObject> core::ops::DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (&mut *self.ptr.as_ptr()).value }
    }
}

impl<T: GcObject> Copy for Handle<T> {}
impl<T: GcObject> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, T: GcObject> core::ops::Deref for Root<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let inner = unsafe { &*self.inner };
        unsafe { &((&*inner.obj.cast::<GcBox<T>>()).value) }
    }
}

impl<T: GcObject> core::ops::DerefMut for Root<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let inner = unsafe { &mut *self.inner };
        unsafe { &mut ((&mut *inner.obj.cast::<GcBox<T>>()).value) }
    }
}
impl<T: GcObject> GcObject for Handle<T> {
    fn visit_references(&self, visit: &mut dyn FnMut(*const GcBox<()>)) {
        visit(self.gc_ptr());
    }
}
impl<T: GcObject> Drop for Root<T> {
    fn drop(&mut self) {
        let inner = unsafe { &mut *self.inner };
        inner.rc = inner.rc.wrapping_sub(1);
    }
}

impl<T: GcObject> Clone for Root<T> {
    fn clone(&self) -> Self {
        let mut inn = unsafe { &mut *self.inner };
        inn.rc += 1;
        Self {
            inner: self.inner,
            _marker: Default::default(),
        }
    }
}

macro_rules! simple {
    ($($t: ty)*) => {
        $(
        impl GcObject for $t {}
        )*
    };
}

simple!(
    u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
    bool
    f32 f64
);

impl<T: GcObject> GcObject for Vec<T> {
    fn visit_references(&self, trace: &mut dyn FnMut(*const GcBox<()>)) {
        for elem in self.iter() {
            elem.visit_references(trace);
        }
    }
}

impl<T: GcObject> GcObject for Option<T> {
    fn visit_references(&self, trace: &mut dyn FnMut(*const GcBox<()>)) {
        if let Some(val) = self {
            val.visit_references(trace);
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct CycleStats {
    marks: usize,
    mismatch_marks: usize,
    sweeped: usize,
    freed: usize,
    remembered: usize,
    promoted: usize,
}

struct CollectionStats {
    collections: usize,
    total_pause: f32,
    pauses: Vec<f32>,
}

impl CollectionStats {
    fn new() -> CollectionStats {
        CollectionStats {
            collections: 0,
            total_pause: 0f32,
            pauses: Vec::new(),
        }
    }

    fn add(&mut self, pause: f32) {
        self.collections += 1;
        self.total_pause += pause;
        self.pauses.push(pause);
    }

    fn pause(&self) -> f32 {
        self.total_pause
    }

    fn pauses(&self) -> AllNumbers {
        AllNumbers(self.pauses.clone())
    }

    fn mutator(&self, runtime: f32) -> f32 {
        runtime - self.total_pause
    }

    fn collections(&self) -> usize {
        self.collections
    }

    fn percentage(&self, runtime: f32) -> (f32, f32) {
        let gc_percentage = ((self.total_pause / runtime) * 100.0).round();
        let mutator_percentage = 100.0 - gc_percentage;

        (mutator_percentage, gc_percentage)
    }
}

pub struct AllNumbers(Vec<f32>);

impl fmt::Display for AllNumbers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        let mut first = true;
        for num in &self.0 {
            if !first {
                write!(f, ",")?;
            }
            write!(f, "{:.1}", num)?;
            first = false;
        }
        write!(f, "]")
    }
}
struct FormattedSize {
    size: usize,
}

impl fmt::Display for FormattedSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ksize = (self.size as f64) / 1024f64;

        if ksize < 1f64 {
            return write!(f, "{}B", self.size);
        }

        let msize = ksize / 1024f64;

        if msize < 1f64 {
            return write!(f, "{:.1}K", ksize);
        }

        let gsize = msize / 1024f64;

        if gsize < 1f64 {
            write!(f, "{:.1}M", msize)
        } else {
            write!(f, "{:.1}G", gsize)
        }
    }
}
fn formatted_size(size: usize) -> FormattedSize {
    FormattedSize { size }
}
