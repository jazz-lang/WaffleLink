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

    pub fn cast<T: CellTrait>(&self) -> &T {
        assert_eq!(self.ty(), T::TYPE);
        unsafe { std::mem::transmute(self) }
    }
    pub fn cast_mut<T: CellTrait>(&mut self) -> &mut T {
        assert_eq!(self.ty(), T::TYPE);
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
    fn visit_references(&self, tracer: &mut Tracer<'_>) {
        match self.ty() {
            CellType::Proto => self.cast::<Proto>().visit_references(tracer),
            CellType::Array => self.cast::<Array>().visit_references(tracer),
            CellType::Function | CellType::Closure => {
                self.cast::<Closure>().visit_references(tracer)
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

impl GcObject for FFIObject {
    fn visit_references(&self, tracer: &mut crate::gc::object::Tracer<'_>) {
        if let Some(trace) = self.trace {
            trace(self.data, tracer)
        }
    }

    fn finalize(&mut self) {
        if let Some(finalize) = self.finalize {
            finalize(self.data);
        }
    }
}

pub trait CellTrait {
    const TYPE: CellType;
}

impl CellTrait for FFIObject {
    const TYPE: CellType = CellType::ComObj;
}
