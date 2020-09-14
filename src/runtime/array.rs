use super::cell_type::*;
use crate::gc::object::*;
use crate::isolate::Isolate;
use crate::values::*;
/// Immutable array value
#[repr(C, align(8))]
pub struct Array {
    ty: CellType,
    length: u32,
    data: u8,
}

impl GcObject for Array {
    fn size(&self) -> usize {
        Array::compute_size(self.length as _)
    }

    fn visit_references(&self, trace: &mut dyn FnMut(*const GcBox<()>)) {
        self.for_each(|value| {
            if value.is_cell() && !value.is_empty() {
                trace(value.as_cell().gc_ptr());
            }
        });
    }
}

impl Array {
    /// Create new array in local scope
    pub fn new_local<'a>(scope: &mut LocalScope, default_init: Value, len: u32) -> Local<'a, Self> {
        let mut val = scope.allocate(Self {
            ty: CellType::Array,
            length: len,
            data: 0,
        });
        val.for_each_mut(|val| {
            *val = default_init;
        });
        val
    }
    /// Allocate new array in Isolate heap.
    pub fn new<'a>(isolate: &mut Isolate, default_init: Value, len: u32) -> Handle<Self> {
        let mut val = isolate.heap().allocate(Self {
            ty: CellType::Array,
            length: len,
            data: 0,
        });
        val.for_each_mut(|val| {
            *val = default_init;
        });
        val
    }
    fn compute_size(len: usize) -> usize {
        core::mem::size_of::<Array>() + (8 * len)
    }
    /// Return raw pointer to array.
    pub fn as_ptr(&self) -> *mut Value {
        (&self.data as *const u8) as *mut Value
    }
    /// Get value at `ix`. If `ix` < self.length returns value
    pub fn at(&self, ix: u32) -> Option<Value> {
        if ix < self.length {
            unsafe { Some(self.as_ptr().offset(ix as _).read()) }
        } else {
            None
        }
    }
    /// Get mutable reference to value at `ix`. If `ix` < self.length returns value
    pub fn at_mut(&mut self, ix: u32) -> Option<&mut Value> {
        if ix < self.length {
            unsafe { Some(&mut *self.as_ptr().offset(ix as _)) }
        } else {
            None
        }
    }
    /// Iterate through each value in array
    pub fn for_each(&self, mut visitor: impl FnMut(Value)) {
        for i in 0..self.length {
            visitor(self.at(i).unwrap());
        }
    }
    /// Iterate through each value in array
    pub fn for_each_mut(&mut self, mut visitor: impl FnMut(&mut Value)) {
        for i in 0..self.length {
            visitor(self.at_mut(i).unwrap())
        }
    }
    /// Convert this array to slice
    pub fn as_slice(&self) -> &[Value] {
        unsafe { std::slice::from_raw_parts(self.as_ptr(), self.length as _) }
    }
    /// Convert this array to mutable slice
    pub fn as_mut_slice(&self) -> &mut [Value] {
        unsafe { std::slice::from_raw_parts_mut(self.as_ptr(), self.length as _) }
    }
}
