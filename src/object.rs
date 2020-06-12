use super::*;
use value::*;
#[repr(C)]
pub struct ObjectHeader {
    pub marked: bool,
}

#[repr(C)]
pub struct Array {
    tag: CellTag,
    mark: bool,
    pub array: Vec<super::value::Value>,
}

pub enum ObjectValue {
    Array(Box<Vec<Value>>),
}

pub struct Object {
    pub(crate) tag: CellTag,
    pub(crate) mark: bool,
    pub table: HashMap<String, Value>,
}

impl Object {
    pub fn visit(&self, trace: &mut impl FnMut(*const Self)) {}
    pub fn finalize(&mut self) {}
}
use std::collections::HashMap;

pub enum CellTag {
    Object,
    Array,
    String,
}

#[repr(C)]
pub struct Cell {
    pub(crate) tag: CellTag,
    pub(crate) mark: bool,
}

impl Cell {
    pub fn visit(&self, trace: &mut impl FnMut(*const Self)) {}
    pub fn size(&self) -> usize {
        match self.tag {
            CellTag::Array => std::mem::size_of::<Array>(),
            CellTag::Object => std::mem::size_of::<Object>(),
            _ => unimplemented!(),
        }
    }
    pub fn finalize(&mut self) {}
}
