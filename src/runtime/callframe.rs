//! Implementation of call frame and call stack for interpreter/JIT
use crate::values::*;

/// CallFrame stores function stack, arguments and some other
/// important things
pub struct CallFrame {
    pub(crate) try_catch: Vec<u32>,
    pub stack: Vec<Value>,
    pub(crate) this: Value,
    pub(crate) arguments: *const Value,
    pub(crate) argc: u32,
    pub(crate) callee: Value,
}
