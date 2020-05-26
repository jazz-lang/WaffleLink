use crate::runtime::*;
use value::*;

pub struct Frame {
    pub stack: Vec<Value>,

    /// 'this' value.
    pub this: Value,
    /// Function environment (captured values)
    pub env: Value,
    /// Current function
    pub func: Value,
    /// Current block pointer
    pub bp: usize,
    /// Current instruction pointer
    pub ip: usize,
    /// Exception handlers
    pub handlers: Vec<u16>,
    /// Leave interpreter.
    pub exit_on_return: bool,
}
