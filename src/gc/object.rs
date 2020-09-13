use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
pub trait GcObject {
    /// Returns size of current object. This is usually just `size_of_val(self)` but in case
    /// when you need dynamically sized type e.g array then this should return something like
    /// `size_of(Base) + length * size_of(Value)`
    fn size(&self) -> usize {
        core::mem::size_of_val(self)
    }

    fn visit_references(&self, trace: &mut dyn FnMut(*const GcBox<()>)) {}
    fn finalize(&mut self) {
        //eprintln!("dead {:p}", self);
    }
}

pub struct Header {
    pub vtable: *mut (),
    pub cell_state: u8,
}

impl Header {
    pub fn cell_state_atomic(&self) -> &AtomicU8 {
        unsafe { &*(&self.cell_state as *const u8 as *const AtomicU8) }
    }
    pub fn vtable_atomic(&self) -> &AtomicUsize {
        unsafe { std::mem::transmute(&self.vtable) }
    }

    pub fn vtable(&self) -> *mut () {
        self.vtable
    }
    pub fn tag(&self) -> u8 {
        (self.vtable as usize & 0x03) as u8
    }
    pub fn cell_state(&self) -> u8 {
        self.cell_state_atomic().load(Ordering::Relaxed)
        //(self.vtable as usize & 0x03) as u8
    }

    pub fn set_vtable(&mut self, vtable: *mut ()) {
        self.vtable = (vtable as usize) as *mut ();
    }
    pub fn set_tag(&mut self, tag: u8) {
        self.vtable = (self.vtable() as usize | tag as usize) as *mut ();
    }
    pub fn set_cell_state(&mut self, tag: u8) {
        self.cell_state_atomic().store(tag, Ordering::Relaxed);
        //self.vtable = (self.vtable() as usize | tag as usize) as *mut ();
    }
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

    pub fn atomic_tag(&self) -> u8 {
        (self.vtable_atomic().load(Ordering::Relaxed) & 0x03) as u8
    }
}

#[repr(C)]
pub struct GcBox<T: GcObject> {
    pub header: Header,
    pub value: T,
}

#[repr(C)]
pub struct TraitObject {
    pub data: *mut (),
    pub vtable: *mut (),
}

impl<T: GcObject> GcBox<T> {
    pub fn is_precise_allocation(&self) -> bool {
        super::precise_allocation::PreciseAllocation::is_precise(self as *const _ as *mut _)
    }

    pub fn precise_allocation(&self) -> *mut super::precise_allocation::PreciseAllocation {
        unsafe {
            super::precise_allocation::PreciseAllocation::from_cell(self as *const _ as *mut _)
        }
    }

    pub fn block(&self) -> *mut super::block::Block {
        super::block::Block::from_cell(super::Address::from_ptr(self))
    }

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
pub struct Handle<T: GcObject> {
    pub(super) ptr: core::ptr::NonNull<GcBox<T>>,
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

pub type GCObjectRef = *mut GcBox<()>;

impl<T: GcObject> GcBox<T> {
    pub fn zap(&mut self, reason: u32) {
        self.header.vtable = 0 as *mut ();
    }

    pub fn is_zapped(&self) -> bool {
        self.header.vtable.is_null()
    }
}

pub struct LocalScopeInner {
    pub gc: *mut dyn super::GarbageCollector,
    pub locals: Vec<*mut GcBox<()>>,
    pub dead: bool,
}

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
    pub fn allocate<'a, T: GcObject + 'a>(&mut self, value: T) -> Local<'a, T> {
        let gc = self.gc();
        let handle = unsafe { super::gc_alloc_handle(gc, value) };
        self.inner().locals.push(handle.ptr.as_ptr().cast());
        Local {
            ptr: self.inner().locals.last_mut().unwrap() as *mut *mut _,
            _marker: Default::default(),
        }
    }

    pub fn new_local<'a, T: GcObject>(&mut self, handle: Handle<T>) -> Local<'a, T> {
        self.inner().locals.push(handle.ptr.as_ptr().cast());
        Local {
            ptr: self.inner().locals.last_mut().unwrap() as *mut *mut _,
            _marker: Default::default(),
        }
    }
}

pub struct Local<'a, T: GcObject> {
    ptr: *mut *mut GcBox<()>,
    _marker: core::marker::PhantomData<&'a T>,
}

impl<'a, T: GcObject> Local<'a, T> {
    pub fn new(scope: &mut LocalScope, value: T) -> Self {
        scope.allocate(value)
    }

    pub fn to_heap(&self) -> Handle<T> {
        Handle {
            ptr: core::ptr::NonNull::new(unsafe { self.ptr.read().cast() }).unwrap(),
        }
    }
}

impl<'a, T: GcObject> Drop for Local<'a, T> {
    fn drop(&mut self) {
        unsafe {
            self.ptr.write(core::ptr::null_mut());
        }
    }
}

impl Drop for LocalScope {
    fn drop(&mut self) {
        self.inner().locals.clear();
        self.inner().dead = true;
    }
}

impl<T: GcObject> std::ops::Deref for Local<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &(&*(&*self.ptr).cast::<GcBox<T>>()).value }
    }
}
impl<T: GcObject> std::ops::DerefMut for Local<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (&mut *(&mut *self.ptr).cast::<GcBox<T>>()).value }
    }
}
