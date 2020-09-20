//! # WaffleLink Cell structure.
//!
//!
//! `Cell` is *base* for all of objects in runtime. It stores cell type and some other important data.

use super::cell_type::CellType;
use super::{array::*, object::*};
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
