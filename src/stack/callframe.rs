use crate::object::Ref;
use crate::value::Value;
#[repr(C)]
pub struct CallFrame {
    pub registers: Box<[Value]>,
    pub this: Value,
    pub env: Value,
    pub module: Value,
    pub arguments: Vec<Value>,
    pub regc: u8,
}

