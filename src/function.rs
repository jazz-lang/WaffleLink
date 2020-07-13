use crate::bytecode::Ins;
use crate::object::*;
use crate::vtable::*;
#[repr(C)]
pub struct Function {
    header: Header,
    pub(crate) func_ptr: usize,
    pub(crate) bc: Option<Vec<Ins>>,
    pub native: bool,
}

impl Function {}

pub static FUNCTION_VTBL: VTable = VTable {
    element_size: 0,
    instance_size: std::mem::size_of::<Function>(),
    parent: None,
    lookup_fn: None,
    index_fn: None,
    calc_size_fn: None,
    apply_fn: None,
    destroy_fn: None,
    trace_fn: None,
    set_fn: None,
    set_index_fn: None,
};
