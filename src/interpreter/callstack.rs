use crate::bytecode::*;
use crate::heap::api::*;
use crate::jit::func::Handler;
use crate::runtime;
use crate::runtime::deref_ptr::DerefPointer;
use runtime::value::*;
use virtual_reg::*;

#[repr(C)]
pub struct CallFrame {
    pub registers: Vec<Value>,
    /// a.k.a arguments
    pub entries: Vec<Value>,
    pub this: Value,
    pub func: Value,
    pub ip: usize,
    pub bp: usize,
    pub code: Handle<CodeBlock>,
    pub handlers: Vec<usize>,
    pub exit_on_return: bool,
    pub rreg: VirtualRegister,
}

impl CallFrame {
    pub fn new(func: Value, code: Handle<CodeBlock>, this: Value) -> Self {
        Self {
            func,
            code,
            registers: vec![Value::undefined(); code.tmp_regs_count as usize],
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
        if r.is_local() {
            self.registers[r.to_local() as usize]
        } else if r.is_argument() && !r.is_constant() {
            self.entries[r.to_argument() as usize]
        } else if r.is_constant() {
            self.code.get_constant(r.to_constant())
        } else {
            unreachable!()
        }
    }

    pub fn r_mut(&mut self, r: VirtualRegister) -> &mut Value {
        if r.is_local() {
            &mut self.registers[r.to_local() as usize]
        } else if r.is_argument() {
            &mut self.entries[r.to_argument() as usize]
        } else if r.is_constant() {
            self.code.get_constant_mut(r.to_constant())
        } else {
            unreachable!()
        }
    }

    pub extern "C" fn push_handler(mut this: Handle<Self>, lbl: usize) {
        this.handlers.push(lbl);
    }

    pub extern "C" fn pop_handler_or_zero(mut this: Handle<Self>) -> usize {
        this.handlers.pop().unwrap_or(0)
    }
    #[no_mangle]
    pub fn test_local(mut this: Handle<Self>, x: i32) -> Value {
        unsafe { this.this }
    }
}

impl Traceable for CallFrame {
    fn trace_with(&self, tracer: &mut Tracer) {
        log::warn!("Trace callframe");
        self.code.trace_with(tracer);
        self.registers.trace_with(tracer);
        self.entries.trace_with(tracer);
        self.func.trace_with(tracer);
        self.this.trace_with(tracer);
    }
}

impl Finalizer for CallFrame {
    fn finalize(&mut self) {
        log::warn!("Fin callframe");
    }
}

pub struct CallStack {
    pub(crate) stack: Vec<StackEntry>,
    pub limit: usize,
}

pub enum StackEntry {
    Frame(CallFrame),
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
        code: Handle<CodeBlock>,
        this: Value,
    ) -> Result<(), Value> {
        if self.stack.len() + 1 >= self.limit {
            return Err(Value::from(
                rt.allocate_string("Maximum call stack size exceeded"),
            ));
        }
        let entry = CallFrame::new(func, code, this);
        self.stack.push(StackEntry::Frame(entry));
        Ok(())
    }

    pub fn pop(&mut self) -> Option<StackEntry> {
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

impl Traceable for CallStack {
    fn trace_with(&self, tracer: &mut Tracer) {
        log::warn!("AAAAA");
        for frame in self.stack.iter() {
            match frame {
                StackEntry::Frame(f) => {
                    f.trace_with(tracer);
                }
                _ => (),
            }
        }
    }
}

impl Finalizer for CallStack {
    fn finalize(&mut self) {}
}
