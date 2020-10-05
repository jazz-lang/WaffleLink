use crate::bytecode::*;
use crate::prelude::*;

pub struct Module {
    pub(crate) ty: CellType,
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
}

impl GcObject for Module {}
impl CellTrait for Module {
    const TYPE: CellType = CellType::Module;
}
