use crate::bytecode::*;
use crate::heap::api::*;
use crate::jit::func::Handler;
use crate::runtime;
use crate::runtime::deref_ptr::DerefPointer;
use runtime::value::*;
use virtual_reg::*;
#[repr(C)]
pub struct CallFrame {
    pub registers: [Value; 64],
    /// a.k.a arguments
    pub entries: Vec<Value>,
    pub this: Value,
    pub func: Value,
    pub ip: usize,
    pub bp: usize,
    pub code: crate::Rc<CodeBlock>,
    pub handlers: Vec<usize>,
    pub exit_on_return: bool,
    pub rreg: VirtualRegister,
}

impl CallFrame {
    pub fn new(func: Value, code: crate::Rc<CodeBlock>, this: Value) -> Self {
        Self {
            func,
            code: code.clone(),
            registers: { [Value::undefined(); 64] },
            entries: vec![Value::undefined(); code.arg_regs_count as usize],
            this,
            ip: 0,
            handlers: vec![],
            bp: 0,
            exit_on_return: false,
            rreg: VirtualRegister::tmp(0),
        }
    }
    pub fn r(&self, r: VirtualRegister) -> Value {
        unsafe {
            if r.is_local() {
                *self.registers.get_unchecked(r.to_local() as usize)
            } else if r.is_argument() && !r.is_constant() {
                *self.entries.get_unchecked(r.to_argument() as usize)
            } else if r.is_constant() {
                self.code.get_constant(r.to_constant())
            } else {
                unreachable!()
            }
        }
    }

    pub fn r_mut(&mut self, r: VirtualRegister) -> &mut Value {
        unsafe {
            if r.is_local() {
                self.registers.get_unchecked_mut(r.to_local() as usize)
            } else if r.is_argument() {
                self.entries.get_unchecked_mut(r.to_argument() as usize)
            } else if r.is_constant() {
                self.code.get_constant_mut(r.to_constant())
            } else {
                unreachable!()
            }
        }
    }

    pub extern "C" fn push_handler(mut this: &mut Self, lbl: usize) {
        this.handlers.push(lbl);
    }

    pub extern "C" fn pop_handler_or_zero(mut this: &mut Self) -> usize {
        let h = this.handlers.pop().unwrap_or(0);

        h
    }
}

pub struct CallStack {
    pub(crate) stack: Vec<StackEntry>,
    pub limit: usize,
}

pub enum StackEntry {
    Frame(Box<CallFrame>),
    Native { func: Value },
}

impl CallStack {
    pub fn new(limit: usize) -> Self {
        Self {
            stack: vec![],
            limit,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn push(
        &mut self,
        rt: &mut runtime::Runtime,
        func: Value,
        code: crate::Rc<CodeBlock>,
        this: Value,
    ) -> Result<(), Value> {
        if self.stack.len() + 1 >= self.limit {
            return Err(Value::from(
                rt.allocate_string("Maximum call stack size exceeded"),
            ));
        }
        let entry = Box::new(CallFrame::new(func, code, this));
        self.stack.push(StackEntry::Frame(entry));
        Ok(())
    }

    pub fn pop(&mut self) -> Option<StackEntry> {
        assert!(self as *const Self as *const u8 as usize >= 1000);
        self.stack.pop()
    }

    pub fn current_frame(&mut self) -> DerefPointer<CallFrame> {
        match self.stack.last() {
            Some(StackEntry::Frame(frame)) => DerefPointer::new(frame),
            None => unreachable!("wtf"),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
