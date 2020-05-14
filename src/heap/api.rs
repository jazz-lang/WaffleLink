use crate::common::Address;
use smallvec::SmallVec;
pub unsafe trait Trace: Finalizer {
    fn mark(&self);
    fn unmark(&self);
    fn references(&self) -> SmallVec<[*const dyn HeapTrait; 64]>;
}

#[derive(Default)]
pub struct Tracer {
    stack: SmallVec<[*const dyn HeapTrait; 64]>,
}
impl Tracer {
    pub fn for_each(&mut self, mut f: impl FnMut(*const dyn HeapTrait)) {
        while let Some(item) = self.stack.pop() {
            f(item);
        }
        //self.stack.into_iter().for_each(|x| f(*x));
    }

    pub fn trace(&mut self, item: *const dyn HeapTrait) {
        self.stack.push(item);
    }
}

pub trait Traceable
where
    Self: Finalizer,
{
    fn trace_with(&self, _: &mut Tracer) {}
}

unsafe impl<T: Traceable> Trace for T {
    fn mark(&self) {
        let mut tracer = Tracer::default();
        self.trace_with(&mut tracer);
        tracer.for_each(|pointer| unsafe { (*pointer).mark() });
    }
    fn unmark(&self) {
        let mut tracer = Tracer::default();
        self.trace_with(&mut tracer);
        tracer.for_each(|pointer| unsafe { (*pointer).unmark() });
    }

    fn references(&self) -> SmallVec<[*const dyn HeapTrait; 64]> {
        let mut tracer = Tracer::default();
        self.trace_with(&mut tracer);
        tracer.stack
    }
}

pub unsafe trait HeapTrait {
    fn mark(&self);
    fn unmark(&self);
    fn slot(&self) -> Address;
    fn get_fwd(&self) -> Address;
    fn set_fwd(&self, _: Address);
    fn copy_to(&self, addr: Address);
    fn addr(&self) -> Address;
    fn inner(&self) -> *mut super::heap::HeapInner<dyn Trace>;

    fn is_marked(&self) -> bool;
}

pub trait Finalizer {
    fn finalize(&mut self) {}
}

macro_rules! simple {
    ($($t: ty)*) => {
        $(
            impl Traceable for $t {}
            impl Finalizer for $t {
                fn finalize(&mut self) {
                    log::warn!("Finalize {}",stringify!($t));
                }
            }
        )*
    };
}

simple!(
    i8
    i16
    i32
    i64
    i128
    u8
    u16
    u32
    u64
    u128
    f64
    f32
    bool
    String
    isize
    usize
    std::fs::File
    std::fs::FileType
    std::fs::Metadata
    std::fs::OpenOptions
    std::io::Stdin
    std::io::Stdout
    std::io::Stderr
    std::io::Error
    std::net::TcpStream
    std::net::TcpListener
    std::net::UdpSocket
    std::net::Ipv4Addr
    std::net::Ipv6Addr
    std::net::SocketAddrV4
    std::net::SocketAddrV6
    std::path::Path
    std::path::PathBuf
    std::process::Command
    std::process::Child
    std::process::ChildStdout
    std::process::ChildStdin
    std::process::ChildStderr
    std::process::Output
    std::process::ExitStatus
    std::process::Stdio
    std::sync::Barrier
    std::sync::Condvar
    std::sync::Once
    std::ffi::CStr
    std::ffi::CString
    &'static str
);

impl<T: Traceable> Traceable for Option<T> {
    fn trace_with(&self, tracer: &mut Tracer) {
        if let Some(item) = self {
            item.trace_with(tracer);
        }
    }
}
impl<T: Traceable> Finalizer for Option<T> {
    fn finalize(&mut self) {
        if let Some(item) = self {
            item.finalize();
        }
    }
}

impl<T: Traceable> Traceable for Vec<T> {
    fn trace_with(&self, tracer: &mut Tracer) {
        for item in self.iter() {
            item.trace_with(tracer);
        }
    }
}

impl<T: HeapTrait + Finalizer + 'static> Traceable for T {
    fn trace_with(&self, tracer: &mut Tracer) {
        tracer.trace(self as *const dyn HeapTrait);
    }
}

impl<T: Finalizer> Finalizer for Vec<T> {
    fn finalize(&mut self) {
        for item in self.iter_mut() {
            item.finalize();
        }
    }
}

pub trait RootedTrait
where
    Self: HeapTrait,
{
    fn is_rooted(&self) -> bool;
    fn references(&self) -> SmallVec<[*const dyn HeapTrait; 64]>;
}

pub struct Rooted<T: Trace + ?Sized> {
    pub(super) inner: *mut RootedInner<T>,
}

impl<T: Trace + ?Sized> Rooted<T> {
    fn inner(&self) -> &mut RootedInner<T> {
        unsafe { &mut *self.inner }
    }
    pub fn to_heap(&self) -> Handle<T> {
        Handle {
            inner: self.inner().inner,
        }
    }
    pub fn get(&self) -> &T {
        unsafe { &(&*self.inner().inner).value }
    }
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut (&mut *self.inner().inner).value }
    }
}

pub(super) struct RootedInner<T: Trace + ?Sized> {
    pub(super) counter: u32,
    pub(super) inner: *mut super::heap::HeapInner<T>,
}
impl<T: Trace + ?Sized> Drop for Rooted<T> {
    #[inline(never)]
    fn drop(&mut self) {
        unsafe {
            assert!(!self.inner.is_null());
            let inner = &mut *self.inner;
            inner.counter = inner.counter.wrapping_sub(1);
        }
    }
}

impl<T: Trace + ?Sized> Clone for Rooted<T> {
    #[inline(never)]
    fn clone(&self) -> Self {
        unsafe {
            let inner = &mut *self.inner;
            inner.counter = inner.counter + 1;
            Rooted { inner: self.inner }
        }
    }
}

unsafe impl<T: Trace + Sized + 'static> HeapTrait for RootedInner<T> {
    fn mark(&self) {
        unsafe {
            (&mut *self.inner).state = true;
        }
    }

    fn unmark(&self) {
        unsafe {
            (&mut *self.inner).mark(false);
        }
    }
    fn get_fwd(&self) -> Address {
        unsafe { (&*self.inner).fwdptr() }
    }

    fn set_fwd(&self, fwd: Address) {
        unsafe {
            (&mut *self.inner).set_fwdptr(fwd);
        }
    }

    fn copy_to(&self, addr: Address) {
        ////debug_assert!(addr.is_non_null() && !self.inner.is_null());
        unsafe {
            std::ptr::copy(
                self.inner as *const u8,
                addr.to_mut_ptr(),
                std::mem::size_of_val(&*self.inner),
            )
        }
    }
    fn slot(&self) -> Address {
        ////debug_assert!(!self.inner.is_null());
        let slot = &self.inner;
        Address::from_ptr(slot)
    }
    fn addr(&self) -> Address {
        Address::from_ptr(self.inner as *const u8)
    }
    fn is_marked(&self) -> bool {
        unsafe { (&*self.inner).is_marked() }
    }
    fn inner(&self) -> *mut super::heap::HeapInner<dyn Trace> {
        self.inner
    }
}

impl<T: Trace + Sized + 'static> RootedTrait for RootedInner<T> {
    fn is_rooted(&self) -> bool {
        !(self.counter == 0)
    }
    fn references(&self) -> SmallVec<[*const dyn HeapTrait; 64]> {
        unsafe { (&*self.inner).value.references() }
    }
}

/// Wraps GC heap pointer.
///
/// GC thing pointers on the heap must be wrapped in a `Handle<T>`
pub struct Handle<T: Trace + ?Sized> {
    pub(super) inner: *mut super::heap::HeapInner<T>,
}
impl<T: Trace + ?Sized> From<Rooted<T>> for Handle<T> {
    fn from(x: Rooted<T>) -> Self {
        unsafe {
            Self {
                inner: (*x.inner).inner,
            }
        }
    }
}

impl<T: Trace + ?Sized> From<&Rooted<T>> for Handle<T> {
    fn from(x: &Rooted<T>) -> Self {
        unsafe {
            Self {
                inner: (*x.inner).inner,
            }
        }
    }
}

impl<T: Trace + ?Sized> Handle<T> {
    pub fn get(&self) -> &T {
        unsafe {
            //debug_assert!(!self.inner.is_null());
            let inner = &*self.inner;
            &inner.value
        }
    }

    /// Returns mutable reference to rooted value
    ///
    /// # Safety
    /// Rust semantics doesn't allow two mutable references at the same time and this function is safe as long as you have only one mutable reference.
    ///
    /// If you want to be 100% sure that you don't have two or more mutable references at the same time please use `Heap<RefCell<T>>`
    ///
    ///
    pub fn get_mut(&mut self) -> &mut T {
        unsafe {
            let inner = &mut *self.inner;
            &mut inner.value
        }
    }
}

unsafe impl<T: Trace + Sized + 'static> HeapTrait for Handle<T> {
    fn copy_to(&self, addr: Address) {
        //debug_assert!(addr.is_non_null() && !self.inner.is_null());
        unsafe {
            std::ptr::copy(
                self.inner as *const u8,
                addr.to_mut_ptr(),
                std::mem::size_of_val(&*self.inner),
            )
        }
    }
    fn mark(&self) {
        unsafe {
            (&mut *self.inner).mark(true);
        }
    }
    fn unmark(&self) {
        unsafe {
            (&mut *self.inner).mark(false);
        }
    }
    fn get_fwd(&self) -> Address {
        unsafe { (&*self.inner).fwdptr() }
    }

    fn set_fwd(&self, fwd: Address) {
        unsafe {
            (&mut *self.inner).set_fwdptr(fwd);
        }
    }
    fn slot(&self) -> Address {
        //debug_assert!(!self.inner.is_null());
        let slot = &self.inner;
        Address::from_ptr(slot)
    }
    fn addr(&self) -> Address {
        Address::from_ptr(self.inner as *const u8)
    }
    fn is_marked(&self) -> bool {
        unsafe { (&*self.inner).is_marked() }
    }
    fn inner(&self) -> *mut super::heap::HeapInner<dyn Trace> {
        self.inner
    }
}
impl<T: Trace> Copy for Handle<T> {}
impl<T: Trace> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

use std::cmp;

impl<T: Trace + PartialOrd> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.get().partial_cmp(other.get())
    }
}

impl<T: Trace + Ord> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.get().cmp(other.get())
    }
}

impl<T: Trace + PartialEq> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get().eq(other.get())
    }
}

impl<T: Trace + Eq> Eq for Handle<T> {}

use std::hash::{Hash, Hasher};

impl<T: Trace + Hash> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

use std::fmt;

impl<T: Trace + fmt::Display> fmt::Display for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}

impl<T: Trace + fmt::Debug> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}

impl<T: Trace + PartialOrd> PartialOrd for Rooted<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.get().partial_cmp(other.get())
    }
}

impl<T: Trace + Ord> Ord for Rooted<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.get().cmp(other.get())
    }
}

impl<T: Trace + PartialEq> PartialEq for Rooted<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get().eq(other.get())
    }
}

impl<T: Trace + Eq> Eq for Rooted<T> {}

impl<T: Trace + Hash> Hash for Rooted<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

impl<T: Trace + fmt::Display> fmt::Display for Rooted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}

impl<T: Trace + fmt::Debug> fmt::Debug for Rooted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.get())
    }
}

impl<T: Trace> Finalizer for Handle<T> {
    fn finalize(&mut self) {}
}

impl<T: Traceable> Finalizer for Rooted<T> {
    fn finalize(&mut self) {
        self.get_mut().finalize();
    }
}

use std::ops::{Deref, DerefMut};

impl<T: Traceable> Deref for Rooted<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}

impl<T: Traceable> DerefMut for Rooted<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

impl<T: Traceable> Deref for Handle<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}

impl<T: Traceable> DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

use std::collections::*;

impl<K: Traceable + Eq, V: Traceable> Traceable for HashMap<K, V> {
    fn trace_with(&self, x: &mut Tracer) {
        for (k, v) in self.iter() {
            k.trace_with(x);
            v.trace_with(x);
        }
    }
}
impl<K, V> Finalizer for HashMap<K, V> {}

impl<K> Finalizer for HashSet<K> {}

impl<K: Traceable + Eq> Traceable for HashSet<K> {
    fn trace_with(&self, x: &mut Tracer) {
        for k in self.iter() {
            k.trace_with(x);
        }
    }
}

impl<T: Traceable> Traceable for LinkedList<T> {
    fn trace_with(&self, t: &mut Tracer) {
        for x in self.iter() {
            x.trace_with(t);
        }
    }
}

impl<T> Finalizer for LinkedList<T> {}
