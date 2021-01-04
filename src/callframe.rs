use crate::heap::*;
use crate::object::Value;

#[repr(C, align(8))]
pub struct CallFrame {
    caller: *mut CallFrame,
    this: Value,
    callee: Value,
    acc: Value,
    nlocals: u32,
    nenv: u32,
    env: *mut Value,
    locals_start: [Value; 0],
}

impl CallFrame {
    pub fn locals(&self) -> &mut [Value] {
        unsafe {
            std::slice::from_raw_parts_mut(self.locals_start.as_ptr() as *mut _, self.nlocals as _)
        }
    }

    pub fn get_env(&self, n: usize) -> Value {
        unsafe { self.env.offset(n as _).read() }
    }

    pub fn set_env(&self, n: usize, val: Value) {
        unsafe {
            self.env.offset(n as _).write(val);
        }
    }

    pub fn get_local(&self, n: usize) -> Value {
        unsafe { self.locals_start.as_ptr().offset(n as _).read() }
    }

    pub fn set_local(&self, n: usize, val: Value) {
        unsafe {
            (self.locals_start.as_ptr() as *mut Value)
                .offset(n as _)
                .write(val);
        }
    }
}

impl HeapObject for CallFrame {
    fn visit_references(&self, tracer: &mut dyn Tracer) {
        // Do not trace env as if there is environment then it will be scanned in `callee`.
        self.callee.visit_references(tracer);
        self.this.visit_references(tracer);
        for local in self.locals() {
            local.visit_references(tracer);
        }
        self.acc.visit_references(tracer);
        unsafe {
            if !self.caller.is_null() {
                (&*self.caller).visit_references(tracer);
            }
        }
    }
    fn heap_size(&self) -> usize {
        std::mem::size_of::<Self>() + (8 * self.nlocals as usize)
    }
}
