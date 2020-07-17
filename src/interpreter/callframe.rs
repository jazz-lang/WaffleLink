use crate::bytecode::*;
use crate::object::Ref;
use crate::value::*;
pub struct CallFrame {
    pub regs: Box<[Value]>,
    pub code_block: Option<Ref<CodeBlock>>,
    pub args: Ref<Value>,
    pub handlers: Vec<u32>,
    pub pc: u32,
}

impl CallFrame {
    pub fn new(args: &[Value], regc: u32) -> Self {
        Self {
            regs: vec![Value::undefined(); regc as usize].into_boxed_slice(),
            args: Ref { ptr: args.as_ptr() },
            code_block: None,
            handlers: vec![],
            pc: 0,
        }
    }

    pub fn get_register(&mut self, r: virtual_register::VirtualRegister) -> Value {
        if r.is_constant() {
            return self
                .code_block
                .unwrap()
                .constants
                .get(r.to_constant_index() as usize)
                .copied()
                .unwrap_or(Value::undefined());
        } else if r.is_argument() {
            return *self.args.offset(r.to_argument() as _);
        } else {
            unsafe { *self.regs.get_unchecked(r.to_local() as usize) }
        }
    }

    pub fn put_register(&mut self, r: virtual_register::VirtualRegister, val: Value) {
        if r.is_local() {
            unsafe {
                *self.regs.get_unchecked_mut(r.to_local() as usize) = val;
            }
        }
    }
}
