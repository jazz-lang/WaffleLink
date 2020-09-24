//! # WaffleLink Cell structure.
//!
//!
//! `Cell` is *base* for all of objects in runtime. It stores cell type and some other important data.
use super::cell_type::CellType;
use super::{array::*, object::*};
use std::sync::Arc;

use crate::isolate::*;
/// Cell definition
#[repr(C)]
pub struct Cell {
    ty: CellType,
}

impl Cell {
    /// Get type of this cell.
    pub const fn ty(&self) -> CellType {
        self.ty
    }

    pub fn cast<T>(&self) -> &T {
        unsafe { std::mem::transmute(self) }
    }
    pub fn cast_mut<T>(&mut self) -> &mut T {
        unsafe { std::mem::transmute(self) }
    }
}

use crate::gc::object::*;

impl Handle<Cell> {
    pub fn cast<T: GcObject>(&self) -> Handle<T> {
        unsafe { std::mem::transmute(self) }
    }
}

impl Local<Cell> {
    pub fn cast<T: GcObject>(&self) -> Local<T> {
        unsafe { std::mem::transmute(self) }
    }
}

impl GcObject for Cell {
    fn visit_references(&self, trace: &mut dyn FnMut(*const *mut GcBox<()>)) {
        match self.ty() {
            CellType::Proto => self.cast::<Proto>().visit_references(trace),
            CellType::Array => self.cast::<Array>().visit_references(trace),
            CellType::Function | CellType::Closure => {
                self.cast::<Closure>().visit_references(trace)
            }
            _ => todo!(),
        }
    }
}

#[repr(C)]
pub struct FFIObject {
    pub(crate) cell_type: CellType,
    pub data: *mut u8,
    pub hash: Option<extern "C" fn(&Arc<Isolate>, u64, *mut u8) -> u64>,
    pub trace: Option<extern "C" fn(*mut u8, &mut Tracer<'_>)>,
    pub finalize: Option<extern "C" fn(*mut u8)>,
}

pub struct Tracer<'a> {
    closure: &'a mut dyn FnMut(*const *mut GcBox<()>),
}

pub extern "C" fn wafflelink_tracer_trace(tracer: &mut Tracer<'_>, obj: &Handle<()>) {
    (tracer.closure)(obj.gc_ptr());
}

impl GcObject for FFIObject {
    fn visit_references(&self, v: &mut dyn FnMut(*const *mut GcBox<()>)) {
        if let Some(trace) = self.trace {
            trace(self.data, &mut Tracer { closure: v })
        }
    }

    fn finalize(&mut self) {
        if let Some(finalize) = self.finalize {
            finalize(self.data);
        }
    }
}
