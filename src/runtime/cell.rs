use super::*;
use transition_map::*;
use value::*;
use vtable::*;
pub struct CellHeader {
    pub tag: u32,
    pub vtable: &'static VTable,
}

#[repr(C)]
pub struct Cell {
    pub header: CellHeader,
}
impl Cell {
    pub fn to<T: CellTy>(&self) -> &mut T {
        assert_eq!(self.header.tag, T::TAG);
        unsafe { std::mem::transmute(self as *const Self as *mut Self) }
    }
    pub fn is<T: CellTy>(&self) -> bool {
        self.header.tag == T::TAG
    }
}
use crate::gc::*;

impl Collectable for Cell {
    fn walk_references(&self, trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        if self.is::<Array>() {
            self.to::<Array>().walk_references(trace);
        } else if self.is::<Object>() {
            self.to::<Object>().walk_references(trace);
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum CellTypes {
    Object = 0,
    Array = 1,

    String = 2,
    Function,
}

#[repr(C)]
pub struct Object {
    pub header: CellHeader,
    pub table: Table,
}

impl Collectable for Object {
    fn walk_references(&self, trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        self.table.walk_references(trace);
    }
}

pub trait CellTy {
    const TAG: u32;
}

#[repr(C)]
pub struct Array {
    pub header: CellHeader,
    pub table: Table,
    pub array: Vec<Value>,
}
impl CellTy for Array {
    const TAG: u32 = CellTypes::Array as _;
}

impl CellTy for Object {
    const TAG: u32 = CellTypes::Object as _;
}
impl Collectable for Array {
    fn walk_references(&self, trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        self.table.walk_references(trace);
        self.array.walk_references(trace);
    }
}
