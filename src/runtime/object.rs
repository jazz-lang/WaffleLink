use super::cell::*;
use super::cell_type::*;
use crate::values::Value;
use crate::gc::object::*;
/// Object type definition
#[repr(C)]
pub struct Object {
    ty: CellType,
}



/// Representation of object properties
#[repr(C)]
pub struct ObjectProperty {
    pub ty: CellType,
    pub key: Value,
    pub value: Value,
    pub hash: u64,
    pub enumerable: bool,
    pub get: Value,
    pub set: Value
}

impl GcObject for ObjectProperty {
    fn visit_references(&self,trace: &mut dyn FnMut(*const GcBox<()>)) {
        if self.key.is_cell() {
            trace(self.key.as_cell().gc_ptr());
        }
        if self.value.is_cell() {
            trace(self.value.as_cell().gc_ptr());
        }
        if self.get.is_cell() {
            trace(self.value.as_cell().gc_ptr());
        }
        if self.set.is_cell() {
            trace(self.value.as_cell().gc_ptr());
        }
    }
}