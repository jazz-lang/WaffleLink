use crate::bytecode::*;
use crate::object::Ref;
use crate::value::*;
pub struct CallFrame {
    pub regs: Box<[Value]>,
    pub code_block: Option<Ref<CodeBlock>>,
    pub args: Ref<Value>,
}

impl CallFrame {
    pub fn new(args: &[Value], regc: u32) -> Self {
        Self {
            regs: vec![Value::undefined(); regc as usize].into_boxed_slice(),
            args: Ref { ptr: args.as_ptr() },
            code_block: None,
        }
    }
}
