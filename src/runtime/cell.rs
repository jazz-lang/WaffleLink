//! # WaffleLink Cell structure.
//!
//!
//! `Cell` is *base* for all of objects in runtime. It stores cell type and some other important data.

use super::cell_type::CellType;

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
    fn visit_references(&self, trace: &mut dyn FnMut(*const GcBox<()>)) {}
}
