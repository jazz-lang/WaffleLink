use crate::utils::segmented_vec::SegmentedVec;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
/// Object that can be handled by GC.
pub trait GcObject {
    /// Returns size of current object. This is usually just `size_of_val(self)` but in case
    /// when you need dynamically sized type e.g array then this should return something like
    /// `size_of(Base) + length * size_of(Value)`
    fn size(&self) -> usize {
        core::mem::size_of_val(self)
    }
    /// Visit all references to GC objects in this object.
    fn visit_references(&self, trace: &mut dyn FnMut(*const GcBox<()>)) {}
    /// Finalization.
    fn finalize(&mut self) {
        //eprintln!("dead {:p}", self);
    }
}
/// GC object header.
pub struct Header {
    /// Pointer to vtable
    pub vtable: *mut (),
    /// Cell state (color)
    pub cell_state: u8,
}

impl Header {
    /// Return atomic pointer to cell state
    pub fn cell_state_atomic(&self) -> &AtomicU8 {
        unsafe { &*(&self.cell_state as *const u8 as *const AtomicU8) }
    }
    /// Return atomic pointer to vtable
    pub fn vtable_atomic(&self) -> &AtomicUsize {
        unsafe { std::mem::transmute(&self.vtable) }
    }
    /// Get vtable
    pub fn vtable(&self) -> *mut () {
        self.vtable
    }
    /// Get vtable tag
    pub fn tag(&self) -> u8 {
        (self.vtable as usize & 0x03) as u8
    }
    /// Load cell state
    pub fn cell_state(&self) -> u8 {
        self.cell_state_atomic().load(Ordering::Relaxed)
        //(self.vtable as usize & 0x03) as u8
    }
    /// Set vtable
    pub fn set_vtable(&mut self, vtable: *mut ()) {
        self.vtable = (vtable as usize) as *mut ();
    }
    /// Set vtable tag
    pub fn set_tag(&mut self, tag: u8) {
        self.vtable = (self.vtable() as usize | tag as usize) as *mut ();
    }
    /// Set cell state
    pub fn set_cell_state(&mut self, tag: u8) {
        self.cell_state_atomic().store(tag, Ordering::Relaxed);
        //self.vtable = (self.vtable() as usize | tag as usize) as *mut ();
    }
    /// Atomically test and set tag.
    pub fn atomic_test_and_set_tag(&mut self, tag: u8) -> bool {
        let entry = self.cell_state_atomic();
        debug_assert!(tag <= super::GC_BLACK);
        loop {
            let mut current = entry.load(Ordering::Relaxed);
            if current == tag {
                return true;
            }
            let new = current as u8 | tag as u8;
            let res = entry.compare_exchange_weak(
                current as _,
                new as _,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
            match res {
                Ok(_) => break,
                _ => (),
            }
        }
        false
    }
    /// Atomically convert cell from white to gray
    pub fn white_to_gray(&self) -> bool {
        let mut current = self.tag();
        let entry = self.cell_state_atomic();
        loop {
            if current != super::GC_WHITE {
                return false;
            }
            if current == super::GC_GRAY {
                return false;
            }

            match entry.compare_exchange_weak(
                current,
                super::GC_GRAY,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(e) => {
                    current = e;
                }
            }
        }
        true
        /*let current = self.tag();
        if current != super::GC_WHITE || current == super::GC_GRAY {
            return false;
        }
        self.cell_state()
            .compare_exchange(
                super::GC_WHITE,
                super::GC_GRAY,
                Ordering::SeqCst,
                Ordering::Relaxed,
            )
            .is_ok()*/
    }
    /// Atomically convert cell from white to blue
    pub fn to_blue(&self) -> bool {
        let current = self.cell_state;
        let entry = self.cell_state_atomic();
        if current == super::GC_BLUE {
            return false;
        }
        entry
            .compare_exchange(
                current,
                super::GC_BLUE,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
    }
    ///  convert cell from white to gray
    pub fn white_to_gray_unsync(&mut self) -> bool {
        let current = self.cell_state;
        if current != super::GC_WHITE {
            return false;
        }
        if current == super::GC_WHITE {
            return false;
        }
        self.cell_state = super::GC_GRAY;
        true
    }
    /// Convert cell from gray to black
    pub fn gray_to_black_unsync(&mut self) -> bool {
        let current = self.cell_state;
        if current != super::GC_GRAY {
            return false;
        }
        if current == super::GC_BLACK {
            return false;
        }
        self.cell_state = super::GC_BLACK;
        true
    }
    /// Atomically convert cell from gray to black
    pub fn gray_to_black(&self) -> bool {
        let mut current = self.tag();
        let entry = self.cell_state_atomic();
        loop {
            if current != super::GC_GRAY {
                return false;
            }
            if current == super::GC_BLACK {
                return false;
            }

            match entry.compare_exchange_weak(
                current,
                super::GC_BLACK,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(e) => {
                    current = e;
                }
            }
        }
        true
        /*let current = self.tag();
        if current != super::GC_GRAY || current == super::GC_BLACK {
            return false;
        }
        self.cell_state()
            .compare_exchange(
                super::GC_GRAY,
                super::GC_BLACK,
                Ordering::SeqCst,
                Ordering::Relaxed,
            )
            .is_ok()*/
    }
    /// Atomically load vtable tag
    pub fn atomic_tag(&self) -> u8 {
        (self.vtable_atomic().load(Ordering::Relaxed) & 0x03) as u8
    }
}

/// Header + GC value.
#[repr(C)]
pub struct GcBox<T: GcObject> {
    /// GC header
    pub header: Header,
    /// Value instance
    pub value: T,
}

/// The representation of a trait object like `&dyn SomeTrait`.
///
/// This struct has the same layout as types like `&dyn SomeTrait` and
/// `Box<dyn AnotherTrait>`.
///
/// `TraitObject` is guaranteed to match layouts, but it is not the
/// type of trait objects (e.g., the fields are not directly accessible
/// on a `&dyn SomeTrait`) nor does it control that layout (changing the
/// definition will not change the layout of a `&dyn SomeTrait`). It is
/// only designed to be used by unsafe code that needs to manipulate
/// the low-level details.
///
/// There is no way to refer to all trait objects generically, so the only
/// way to create values of this type is with functions like
/// [`std::mem::transmute`][transmute]. Similarly, the only way to create a true
/// trait object from a `TraitObject` value is with `transmute`.
///
/// [transmute]: ../intrinsics/fn.transmute.html
///
/// Synthesizing a trait object with mismatched types—one where the
/// vtable does not correspond to the type of the value to which the
/// data pointer points—is highly likely to lead to undefined
/// behavior.
///

#[repr(C)]
pub struct TraitObject {
    /// pointer to data
    pub data: *mut (),
    /// pointer to vtable
    pub vtable: *mut (),
}

impl<T: GcObject> GcBox<T> {
    /// Return true if this object is precie allocation
    pub fn is_precise_allocation(&self) -> bool {
        super::precise_allocation::PreciseAllocation::is_precise(self as *const _ as *mut _)
    }
    /// Return precise allocation from this object
    pub fn precise_allocation(&self) -> *mut super::precise_allocation::PreciseAllocation {
        super::precise_allocation::PreciseAllocation::from_cell(self as *const _ as *mut _)
    }
    /// Return block where this cell was allocation
    pub fn block(&self) -> *mut super::block::Block {
        super::block::Block::from_cell(super::Address::from_ptr(self))
    }
    /// Return trait object
    pub fn trait_object(&self) -> &mut dyn GcObject {
        unsafe {
            core::mem::transmute(TraitObject {
                data: &self.value as *const _ as *mut _,
                vtable: self.header.vtable(),
            })
        }
    }
}

use std::hash::{Hash, Hasher};

impl GcObject for () {}
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
/// Copy-able pointer to GC object.This struct *must* be reachable
/// from locals or some other roots.
pub struct Handle<T: GcObject> {
    pub(super) ptr: core::ptr::NonNull<GcBox<T>>,
}

impl<T: GcObject> Handle<T> {
    /// Return raw gc ptr
    pub fn gc_ptr(&self) -> *mut GcBox<()> {
        self.ptr.cast::<_>().as_ptr()
    }
    /// Compare two handles by pointer
    pub fn ptr_eq(this: Self, other: Self) -> bool {
        this.ptr == other.ptr
    }
    /// Create from raw
    pub unsafe fn from_raw<U>(x: *const U) -> Self {
        Self {
            ptr: core::ptr::NonNull::new((x as *mut U).cast()).unwrap(),
        }
    }
}

impl<T: GcObject> GcObject for Handle<T> {
    fn visit_references(&self, trace: &mut dyn FnMut(*const GcBox<()>)) {
        trace(self.gc_ptr());
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

    fn finalize(&mut self) {
        eprintln!("ded vec {:p},raw parts={:p}", self, self.as_ptr());
    }
}

impl<T: GcObject> GcObject for Option<T> {
    fn visit_references(&self, trace: &mut dyn FnMut(*const GcBox<()>)) {
        if let Some(val) = self {
            val.visit_references(trace);
        }
    }
}
/// Alias to *mut GcBox<()>
pub type GCObjectRef = *mut GcBox<()>;

impl<T: GcObject> GcBox<T> {
    /// Zap object
    pub fn zap(&mut self, reason: u32) {
        self.header.vtable = 0 as *mut ();
    }
    /// Check if object is zapped
    pub fn is_zapped(&self) -> bool {
        self.header.vtable.is_null()
    }
}
use crate::utils::linked_list::LinkedList;
/// Local scope inner
pub struct LocalScopeInner {
    pub prev: *mut Self,
    pub next: *mut Self,
    /// pointer to gc
    pub gc: *mut dyn super::GarbageCollector,
    /// all locals
    pub locals: LinkedList<*mut GcBox<()>>,
    /// is this scope dead?
    pub dead: bool,
}
/// A stack-allocated structure that governs a number of local handles.
/// After a local scope has been created, all local handles will be
/// allocated within that local scope until either the local scope is
/// deleted or another local scope is created.  If there is already a
/// local scope and a new one is created, all allocations will take
/// place in the new local scope until it is deleted.  After that,
/// new handles will again be allocated in the original local scope.
///
/// After the local scope of a local handle has been deleted the
/// garbage collector will no longer track the object stored in the
/// handle and may deallocate it.  The behavior of accessing a handle
/// for which the local scope has been deleted is undefined.
pub struct LocalScope {
    pub(crate) inner: *mut LocalScopeInner,
}

impl LocalScope {
    fn inner(&self) -> &mut LocalScopeInner {
        unsafe { &mut *self.inner }
    }
    fn gc(&self) -> &mut dyn super::GarbageCollector {
        unsafe { &mut *self.inner().gc }
    }
    /// Allocate `value` in GC heap and return local handle
    pub fn allocate<'a, T: GcObject + 'a>(&mut self, value: T) -> Local<T> {
        let gc = self.gc();
        let handle = unsafe { super::gc_alloc_handle(gc, value) };

        self.inner().locals.push_back(handle.ptr.as_ptr().cast());
        Local {
            ptr: self.inner().locals.back_mut().unwrap() as *mut *mut _,
            _marker: Default::default(),
        }
    }
    /// New local from handle
    pub fn new_local<'a, T: GcObject>(&mut self, handle: Handle<T>) -> Local<T> {
        self.inner().locals.push_back(handle.ptr.as_ptr().cast());
        Local {
            ptr: self.inner().locals.back_mut().unwrap() as *mut *mut _,
            _marker: Default::default(),
        }
    }
}

pub struct UndropLocalScope {
    pub(crate) inner: *mut LocalScopeInner,
}

impl UndropLocalScope {
    fn inner(&self) -> &mut LocalScopeInner {
        unsafe { &mut *self.inner }
    }
    fn gc(&self) -> &mut dyn super::GarbageCollector {
        unsafe { &mut *self.inner().gc }
    }
    /// Allocate `value` in GC heap and return local handle
    pub fn allocate<'a, T: GcObject + 'a>(&mut self, value: T) -> Local<T> {
        let gc = self.gc();
        let handle = unsafe { super::gc_alloc_handle(gc, value) };

        self.inner().locals.push_back(handle.ptr.as_ptr().cast());
        Local {
            ptr: self.inner().locals.back_mut().unwrap() as *mut *mut _,
            _marker: Default::default(),
        }
    }
    /// New local from handle
    pub fn new_local<'a, T: GcObject>(&mut self, handle: Handle<T>) -> Local<T> {
        self.inner().locals.push_back(handle.ptr.as_ptr().cast());
        Local {
            ptr: self.inner().locals.back_mut().unwrap() as *mut *mut _,
            _marker: Default::default(),
        }
    }
}
/// An object reference managed by the WaffleLink garbage collector.
///
/// All objects returned from WaffleLink have to be tracked by the garbage
/// collector so that it knows that the objects are still alive.  Also,
/// because some of the garbage collectors may move objects, it is unsafe to
/// point directly to an object.  Instead, all objects are stored in
/// handles which are known by the garbage collector and updated
/// whenever an object moves.  Handles should always be passed by value
/// (except in cases like out-parameters) and they should never be
/// allocated on the heap.
///
/// There are two types of handles: local and persistent handles.
///
/// Local handles are light-weight and transient and typically used in
/// local operations.  They are managed by LocalScopes. That means that a
/// LocalScope must exist on the stack when they are created and that they are
/// only valid inside of the `LocalScope` active during their creation.
/// For passing a local handle to an outer `LocalScope`, an
/// `EscapableLocalScope` and its `Escape()` method must be used.
///
/// Persistent handles can be used when storing objects across several
/// independent operations and have to be explicitly deallocated when they're no
/// longer used.
///
/// It is safe to extract the object stored in the handle by
/// dereferencing the handle (for instance, to extract the &i32 from
/// a Local<i32>); the value will still be governed by a handle
/// behind the scenes and the same rules apply to these values as to
/// their handles.

pub struct Local<T: GcObject> {
    ptr: *mut *mut GcBox<()>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: GcObject> Local<T> {
    /// Allocate `value` in GC heap and return local handle
    pub fn new(scope: &mut LocalScope, value: T) -> Self {
        scope.allocate(value)
    }
    /// Convert to heap handle.
    pub fn to_heap(&self) -> Handle<T> {
        unsafe {
            let ptr = *self.ptr;
            Handle {
                ptr: core::ptr::NonNull::new(ptr.cast()).unwrap(),
            }
        }
    }
}

impl<'a, T: GcObject> Drop for Local<T> {
    fn drop(&mut self) {
        unsafe {
            self.ptr.write(core::ptr::null_mut());
        }
    }
}

impl Drop for LocalScope {
    fn drop(&mut self) {
        unsafe {
            let prev = self.inner().prev;
            let next = self.inner().next;
            if !prev.is_null() {
                (&mut *prev).next = next;
            }
            if !next.is_null() {
                (&mut *next).prev = prev;
            }
        }
        self.inner().locals.clear();
        self.inner().dead = true;
    }
}

impl<T: GcObject> std::ops::Deref for Local<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &(&*(&*self.ptr).cast::<GcBox<T>>()).value }
    }
}
impl<T: GcObject> std::ops::DerefMut for Local<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (&mut *(&mut *self.ptr).cast::<GcBox<T>>()).value }
    }
}

pub struct Buffer {
    locals: [*mut GcBox<()>; 32],
    ix: usize,
}
