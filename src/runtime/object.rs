use super::cell::*;
use super::cell_type::*;

/// Object type definition
#[repr(C)]
pub struct Object {
    ty: CellType,
}
