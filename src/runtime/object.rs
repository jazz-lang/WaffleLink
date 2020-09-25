use super::cell_type::*;
use crate::bytecode::*;
use crate::gc::object::*;
use crate::isolate::*;
use crate::values::Value;
use std::sync::Arc;
#[repr(C)]
pub struct Proto {
    pub cell_type: CellType,
    pub nstack: u32,
    pub argc: u32,
    pub constants: Vec<Value>,
    pub code: Vec<Op>,
    pub upvaldesc: Vec<UpvalDesc>,
}

pub struct UpvalDesc {
    instack: u16,
    idx: u8,
}
impl GcObject for Proto {
    fn visit_references(&self, tracer: &mut Tracer<'_>) {
        //self.ptab.visit_references(trace);
        self.constants.visit_references(tracer);
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
    fn visit_references(&self, tracer: &mut Tracer<'_>) {
        unsafe {
            (&*self.value).visit_references(tracer);
            match &self.kind {
                UpvalueKind::Next(next) => next.visit_references(tracer),
                UpvalueKind::Closed(value) => value.visit_references(tracer),
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
    fn visit_references(&self, tracer: &mut Tracer<'_>) {
        self.upvals.visit_references(tracer);
        self.proto.visit_references(tracer);
    }
}

#[repr(C)]
pub struct NativeClosure {
    pub cell_type: CellType,
    pub addr: NativeFunc,
    pub argc: i32,
}

pub type NativeFunc = fn(&Arc<Isolate>) -> Result<Value, Value>;
