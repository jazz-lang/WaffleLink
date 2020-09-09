use crate::gc::TaggedPointer;
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
    pub vtable: *mut (),
    pub next: *mut GcBox<()>,
}

impl Header {
    pub fn vtable(&self) -> *mut () {
        (self.vtable as usize & (!0x03)) as *mut _
    }

    pub fn tag(&self) -> u8 {
        (self.vtable as usize & 0x03) as u8
    }

    pub fn set_vtable(&mut self, vtable: *mut ()) {
        self.vtable = (vtable as usize | self.tag() as usize) as *mut ();
    }

    pub fn set_tag(&mut self, tag: u8) {
        self.vtable = (self.vtable() as usize | tag as usize) as *mut ();
    }

    pub fn next(&self) -> *mut () {
        (self.next as usize & (!0x03)) as *mut _
    }

    pub fn next_tag(&self) -> u8 {
        (self.next as usize & 0x03) as u8
    }

    pub fn set_next(&mut self, next: *mut ()) {
        self.next = (next as usize | self.next_tag() as usize) as *mut _;
    }

    pub fn set_next_tag(&mut self, tag: u8) {
        self.next = (self.next() as usize | tag as usize) as *mut _;
    }
}

#[repr(C, packed)]
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
    pub fn trait_object(&self) -> &mut dyn GcObject {
        unsafe {
            core::mem::transmute(TraitObject {
                data: &self.value as *const _ as *mut _,
                vtable: self.header.vtable(),
            })
        }
    }
}

impl<T: GcObject> GcObject for Root<T> {
    fn visit_references(&self, trace: &mut dyn FnMut(*const GcBox<()>)) {
        self.to_heap().visit_references(trace);
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

pub type GCObjectRef = *mut GcBox<()>;

impl<T: GcObject> GcBox<T> {
    pub fn zap(&mut self, reason: u32) {
        self.header.vtable = 0 as *mut ();
    }

    pub fn is_zapped(&self) -> bool {
        self.header.vtable.is_null()
    }
}
