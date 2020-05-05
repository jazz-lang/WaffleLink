pub mod cell;
pub mod deref_ptr;

pub mod pure_nan;
pub mod value;
use cell::*;
use cgc::api::*;
use cgc::heap::Heap;
use std::collections::HashMap;
use value::*;

pub struct Runtime {
    pub heap: Heap,
    pub string_prototype: Value,
    pub object_prototype: Value,
    pub array_prototype: Value,
    pub number_prototype: Value,
    pub function_prototype: Value,
    pub generator_prototype: Value,
    pub process_prototype: Value,
    pub file_prototype: Value,
    pub module_prototype: Value,
    pub boolean_prototype: Value,
    pub byte_array_prototype: Value,
    pub globals: HashMap<String, Value>,
}

impl Runtime {
    #[inline]
    pub fn allocate_cell(&mut self, cell: Cell) -> Rooted<Cell> {
        self.heap.allocate(cell)
    }
    #[inline]
    /// Make some value rooted.
    pub fn make_rooted<T: Traceable + 'static>(&mut self, value: Handle<T>) -> Rooted<T> {
        self.heap.root(value)
    }
    #[inline]
    pub fn allocate<T: Traceable + 'static>(&mut self, val: T) -> Rooted<T> {
        self.heap.allocate(val)
    }

    pub fn allocate_string(&mut self, string: impl AsRef<str>) -> Rooted<Cell> {
        let s = string.as_ref().to_string();
        let proto = self.string_prototype.as_cell();
        let cell = Cell::new(CellValue::String(Box::new(s)), Some(proto));

        self.allocate_cell(cell)
    }
}
