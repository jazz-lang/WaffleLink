use crate::bytecode::Ins;
use crate::object::Header;
#[repr(C)]
pub struct Function {
    header: Header,
    pub(crate) argc_used: u32,
    pub(crate) regs_used: u8,
    pub(crate) bc: Vec<Ins>,
}

impl Function {}
