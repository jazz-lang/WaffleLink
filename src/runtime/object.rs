use super::cell::*;
use super::cell_type::*;
use crate::gc::object::*;
use crate::values::Value;
/// Object type definition
#[repr(C)]
pub struct Class {
    ty: CellType,
}

/// Representation of object properties
#[repr(C)]
pub struct ClassProperty {
    pub ty: CellType,
    pub key: Value,
    pub value: Value,
    pub hash: u64,
    pub enumerable: bool,
    pub get: Value,
    pub set: Value,
}

impl GcObject for ClassProperty {
    fn visit_references(&self, trace: &mut dyn FnMut(*const *mut GcBox<()>)) {
        if self.key.is_cell() {
            trace(self.key.as_cell_ref().gc_ptr());
        }
        if self.value.is_cell() {
            trace(self.value.as_cell_ref().gc_ptr());
        }
        if self.get.is_cell() {
            trace(self.value.as_cell_ref().gc_ptr());
        }
        if self.set.is_cell() {
            trace(self.value.as_cell_ref().gc_ptr());
        }
    }
}
