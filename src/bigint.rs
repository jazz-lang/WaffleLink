use crate::object::*;
use crate::vtable::*;
pub static BIGINT_VTBL: VTable = VTable {
    element_size: 0,
    instance_size: std::mem::size_of::<BigIntObject>(),
    parent: None,
    lookup_fn: None,
    index_fn: None,
    calc_size_fn: None,
    apply_fn: None,
    destroy_fn: None,
    set_fn: None,
    set_index_fn: None,
};

#[repr(C)]
pub struct BigIntObject {
    pub header: Header,
    pub bigint: num_bigint::BigInt,
}
