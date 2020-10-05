use crate::prelude::*;
use std::sync::Arc;
#[repr(C)]
pub struct Proto {
    pub cell_type: CellType,
    pub nstack: u32,
    pub argc: i32,
    pub name: Value, //pub upvaldesc: Vec<UpvalDesc>,
}

impl CellTrait for Proto {
    const TYPE: CellType = CellType::Proto;
}

pub struct UpvalDesc {
    instack: u16,
    idx: u8,
}
impl GcObject for Proto {
    fn visit_references(&self, tracer: &mut Tracer<'_>) {
        //self.ptab.visit_references(trace);
        self.name.visit_references(tracer);
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
    pub upvals: Vec<Value>,
    pub proto: Handle<Proto>,
}

impl CellTrait for Closure {
    const TYPE: CellType = CellType::Closure;
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
    pub name: Value,
    pub argc: i32,
}

impl CellTrait for NativeClosure {
    const TYPE: CellType = CellType::NativeClosure;
}

pub type NativeFunc = fn(&Arc<Isolate>) -> Result<Value, Value>;

impl GcObject for NativeClosure {
    fn visit_references(&self, tracer: &mut Tracer<'_>) {
        self.name.visit_references(tracer);
    }
}
