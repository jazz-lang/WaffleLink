use super::cell::*;
use super::cell_type::*;
use crate::gc::object::*;
use crate::isolate::*;
use crate::values::Value;

#[repr(C)]
pub struct Proto {
    pub cell_type: CellType,
    pub nstack: u8,
    pub argc: u8,
    pub constants: Vec<Value>,
    pub ptab: Vec<Handle<Proto>>,
    pub code: Vec<u32>,
    pub upvaldesc: Vec<UpvalDesc>,
}

pub struct UpvalDesc {
    instack: u16,
    idx: u8,
}
impl GcObject for Proto {
    fn visit_references(&self, trace: &mut dyn FnMut(*const *mut GcBox<()>)) {
        self.ptab.visit_references(trace);
        self.constants.visit_references(trace);
    }
}

pub struct Upvalue {
    pub value: *mut Value,
    pub kind: UpvalueKind,
}

pub enum UpvalueKind {
    Next(Option<Handle<Upvalue>>),
    Closed(Value),
}

impl GcObject for Upvalue {
    fn visit_references(&self, trace: &mut dyn FnMut(*const *mut GcBox<()>)) {
        unsafe {
            (&*self.value).visit_references(trace);
            match &self.kind {
                UpvalueKind::Next(next) => next.visit_references(trace),
                UpvalueKind::Closed(value) => value.visit_references(trace),
            }
        }
    }
}

#[repr(C)]
pub struct Closure {
    pub cell_type: CellType,
    pub upvals: Vec<Upvalue>,
    pub proto: Handle<Proto>,
}

impl GcObject for Closure {
    fn visit_references(&self, trace: &mut dyn FnMut(*const *mut GcBox<()>)) {
        self.upvals.visit_references(trace);
        self.proto.visit_references(trace);
    }
}

#[repr(C)]
pub struct NativeClosure {
    pub cell_type: CellType,
    pub addr: usize,
    ///
    ///
    /// - argc from 0 to 5: pass parameters in CPU registers.
    /// - argc is -1 or bigger than 5: pass parameters in array.
    ///
    pub argc: i32,
}

pub type NativeFunc0 = fn(&mut Isolate) -> Result<Value, Value>;
pub type NativeFunc1 = fn(&mut Isolate, Value) -> Result<Value, Value>;
pub type NativeFunc2 = fn(&mut Isolate, Value, Value) -> Result<Value, Value>;
pub type NativeFunc3 = fn(&mut Isolate, Value, Value, Value) -> Result<Value, Value>;
pub type NativeFunc4 = fn(&mut Isolate, Value, Value, Value, Value) -> Result<Value, Value>;
pub type NativeFunc5 = fn(&mut Isolate, Value, Value, Value, Value, Value) -> Result<Value, Value>;
pub type NativeFuncVaArg = fn(&mut Isolate, &[Value]) -> Result<Value, Value>;
