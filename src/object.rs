use crate::builtins::*;
use crate::gc::*;
use crate::value::*;
use crate::vtable::VTable;
use std;
use std::cmp;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::slice;
use std::str;
use std::sync::atomic::{AtomicUsize, Ordering};
pub struct Header {
    vtable: AtomicUsize,
    fwdptr: AtomicUsize,
}
const MARK_BITS: usize = 2;
const MARK_MASK: usize = (2 << MARK_BITS) - 1;
const FWD_MASK: usize = !0 & !MARK_MASK;
impl Header {
    #[cfg(test)]
    const fn new() -> Header {
        Header {
            vtable: AtomicUsize::new(0),
            fwdptr: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    pub const fn size() -> i32 {
        std::mem::size_of::<Header>() as i32
    }
    #[inline(always)]
    pub fn vtbl(&self) -> &mut VTable {
        unsafe { &mut *self.vtblptr().to_mut_ptr::<VTable>() }
    }

    #[inline(always)]
    pub fn vtblptr(&self) -> Address {
        self.vtable.load(Ordering::Relaxed).into()
    }

    #[inline(always)]
    pub fn set_vtblptr(&mut self, addr: Address) {
        self.vtable.store(addr.to_usize(), Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn vtblptr_forward(&mut self, address: Address) {
        self.vtable.store(address.to_usize() | 1, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn vtblptr_forwarded(&self) -> Option<Address> {
        let addr = self.vtable.load(Ordering::Relaxed);

        if (addr & 1) == 1 {
            Some((addr & !1).into())
        } else {
            None
        }
    }

    pub fn vtblptr_repair(&mut self) {
        let addr = self.vtable.load(Ordering::Relaxed);

        if (addr & 3) == 3 {
            // forwarding failed
            let vtblptr = (addr & !3).into();
            self.set_vtblptr(vtblptr);
        } else if (addr & 1) == 1 {
            // object was forwarded
            let fwd: Address = (addr & !1).into();
            let fwd = unsafe { &*fwd.to_mut_ptr::<Obj>() };
            let vtblptr = fwd.header().vtblptr();

            self.set_vtblptr(vtblptr);
        } else {
            // nothing to do
        }
    }

    #[inline(always)]
    pub fn vtblptr_forwarded_atomic(&self) -> Result<Address, Address> {
        let addr = self.vtable.load(Ordering::Relaxed);

        if (addr & 3) == 3 {
            Ok(Address::from_ptr(self as *const _))
        } else if (addr & 1) == 1 {
            Ok((addr & !1).into())
        } else {
            Err(addr.into())
        }
    }

    #[inline(always)]
    pub fn vtblptr_forward_atomic(
        &mut self,
        expected_vtblptr: Address,
        new_address: Address,
    ) -> Result<(), Address> {
        let fwd = new_address.to_usize() | 1;
        let result =
            self.vtable
                .compare_and_swap(expected_vtblptr.to_usize(), fwd, Ordering::SeqCst);

        if result == expected_vtblptr.to_usize() {
            Ok(())
        } else {
            // If update fails, this needs to be a forwarding pointer
            debug_assert!((result | 1) != 0);

            if (result & 3) == 3 {
                Err(Address::from_ptr(self as *const _))
            } else {
                Err((result & !1).into())
            }
        }
    }

    #[inline(always)]
    pub fn vtblptr_forward_failure_atomic(
        &mut self,
        expected_vtblptr: Address,
    ) -> Result<(), Address> {
        let fwd = expected_vtblptr.to_usize() | 3;
        let result =
            self.vtable
                .compare_and_swap(expected_vtblptr.to_usize(), fwd, Ordering::SeqCst);

        if result == expected_vtblptr.to_usize() {
            Ok(())
        } else {
            // If update fails, this needs to be a forwarding pointer
            debug_assert!((result | 1) != 0);

            if (result & 3) == 3 {
                Err(Address::from_ptr(self as *const _))
            } else {
                Err((result & !1).into())
            }
        }
    }

    #[inline(always)]
    pub fn clear_fwdptr(&self) {
        self.fwdptr.store(0, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn fwdptr_non_atomic(&self) -> Address {
        let fwdptr = self.fwdptr.load(Ordering::Relaxed);
        (fwdptr & FWD_MASK).into()
    }

    #[inline(always)]
    pub fn set_fwdptr_non_atomic(&mut self, addr: Address) {
        debug_assert!((addr.to_usize() & MARK_MASK) == 0);
        let fwdptr = self.fwdptr.load(Ordering::Relaxed);
        self.fwdptr
            .store(addr.to_usize() | (fwdptr & MARK_MASK), Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn mark_non_atomic(&mut self) {
        let fwdptr = self.fwdptr.load(Ordering::Relaxed);
        self.fwdptr.store(fwdptr | 1, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn unmark_non_atomic(&mut self) {
        let fwdptr = self.fwdptr.load(Ordering::Relaxed);
        self.fwdptr.store(fwdptr & FWD_MASK, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn is_marked_non_atomic(&self) -> bool {
        let fwdptr = self.fwdptr.load(Ordering::Relaxed);
        (fwdptr & MARK_MASK) != 0
    }

    #[inline(always)]
    pub fn try_mark_non_atomic(&self) -> bool {
        let fwdptr = self.fwdptr.load(Ordering::Relaxed);

        if (fwdptr & MARK_MASK) != 0 {
            return false;
        }

        self.fwdptr.store(fwdptr | 1, Ordering::Relaxed);
        true
    }

    #[inline(always)]
    pub fn try_mark(&self) -> bool {
        let old = self.fwdptr.load(Ordering::Relaxed);
        self.fwdptr
            .compare_exchange(old, old | 1, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
    }
}

#[repr(C)]
pub struct Obj {
    header: Header,
}

#[repr(C)]
pub struct Ref<T> {
    pub ptr: *const T,
}

unsafe impl<T> Send for Ref<T> {}
unsafe impl<T> Sync for Ref<T> {}

impl<T> Ref<T> {
    pub fn null() -> Ref<T> {
        Ref { ptr: ptr::null() }
    }

    pub fn cast<R>(&self) -> Ref<R> {
        Ref {
            ptr: self.ptr as *const R,
        }
    }

    pub fn raw(&self) -> *const T {
        self.ptr
    }

    pub fn address(&self) -> Address {
        Address::from_ptr(self.ptr)
    }
}

// known limitation of #[derive(Copy, Clone)]
// traits need to be implemented manually
impl<T> Copy for Ref<T> {}
impl<T> Clone for Ref<T> {
    fn clone(&self) -> Ref<T> {
        *self
    }
}

impl<T> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for Ref<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.ptr as *mut T) }
    }
}

impl<T> Into<Ref<T>> for usize {
    fn into(self) -> Ref<T> {
        Ref {
            ptr: self as *const T,
        }
    }
}

impl<T> Into<Ref<T>> for Address {
    fn into(self) -> Ref<T> {
        Ref { ptr: self.to_ptr() }
    }
}
impl Obj {
    #[inline(always)]
    pub fn address(&self) -> Address {
        Address::from_ptr(self as *const _)
    }

    #[inline(always)]
    pub fn header(&self) -> &Header {
        &self.header
    }

    #[inline(always)]
    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }
    pub fn cast<T>(&self) -> Ref<T> {
        unsafe { std::mem::transmute(self) }
    }
    #[inline(always)]
    pub fn data(&self) -> *const u8 {
        unsafe { (self as *const Self as *const u8).offset(Header::size() as _) }
    }

    pub fn is_array_ref(&self) -> bool {
        self.header().vtbl().is_array_ref()
    }

    pub fn size_for_vtblptr(&self, vtblptr: Address) -> usize {
        let vtbl = unsafe { &*vtblptr.to_mut_ptr::<VTable>() };
        let instance_size = vtbl.instance_size;

        if instance_size != 0 {
            return instance_size;
        }
        if vtbl.is_array_ref() {
            determine_array_size(self)
        } else {
            if let Some(c) = vtbl.calc_size_fn {
                c(Ref {
                    ptr: self as *const _,
                })
            } else {
                panic!("Can't determine object size");
            }
        }
    }

    pub fn size(&self) -> usize {
        self.size_for_vtblptr(self.header().vtblptr())
    }

    /*pub fn visit_reference_fields<F>(&mut self, f: F)
    where
        F: FnMut(Slot),
    {
        let classptr = self.header().vtbl().classptr;
        let cls = unsafe { &*classptr };

        visit_refs(self.address(), cls, None, f);
    }

    pub fn visit_reference_fields_within<F>(&mut self, range: Region, f: F)
    where
        F: FnMut(Slot),
    {
        let classptr = self.header().vtbl().classptr;
        let cls = unsafe { &*classptr };

        visit_refs(self.address(), cls, Some(range), f);
    }
    */
    pub fn copy_to(&self, dest: Address, size: usize) {
        unsafe {
            std::ptr::copy(
                self as *const Obj as *const u8,
                dest.to_mut_ptr::<u8>(),
                size,
            );
        }
    }
}
fn determine_array_size(obj: &Obj) -> usize {
    let handle: Ref<Array> = Ref {
        ptr: obj as *const Obj as *const Array,
    };

    let calc = Header::size() as usize
        + std::mem::size_of::<usize>()
        + std::mem::size_of::<Value>() * handle.len() as usize;
    calc
}

#[repr(C)]
pub struct Array {
    header: Header,
    length: usize,
    data: Value,
}

impl Array {
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn data(&self) -> *const Value {
        &self.data as *const Value
    }

    pub fn data_address(&self) -> Address {
        Address::from_ptr(self.data())
    }

    pub fn data_mut(&mut self) -> *mut Value {
        &self.data as *const Value as *mut Value
    }

    pub fn get_at(&self, idx: usize) -> Value {
        unsafe { *self.data().offset(idx as isize) }
    }

    pub fn set_at(&mut self, idx: usize, val: Value) {
        unsafe {
            *self.data_mut().offset(idx as isize) = val;
        }
    }
}
