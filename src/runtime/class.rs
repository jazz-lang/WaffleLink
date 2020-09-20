use super::cell_type::CellType;
use super::object::*;
use crate::gc::object::*;
use crate::values::*;

#[repr(C)]
pub struct Class {
    pub cell_type: CellType,
    pub name: Handle<String>,
    pub super_class: Option<Handle<Self>>,
    pub nmembers: u32,
}

impl GcObject for Class {
    fn visit_references(&self, trace: &mut dyn FnMut(*const *mut GcBox<()>)) {
        self.name.visit_references(trace);
        self.super_class.visit_references(trace);
    }
}

#[repr(C)]
pub struct Instance {
    pub cell_type: CellType,
    pub super_: Option<Handle<Self>>,
    pub class: Handle<Class>,
    pub members: Value,
}

impl Instance {
    pub fn members(&self) -> &[Value] {
        unsafe { std::slice::from_raw_parts(&self.members, self.class.nmembers as _) }
    }

    pub fn members_mut(&mut self) -> &mut [Value] {
        unsafe { std::slice::from_raw_parts_mut(&mut self.members, self.class.nmembers as _) }
    }
}

impl GcObject for Instance {
    fn visit_references(&self, trace: &mut dyn FnMut(*const *mut GcBox<()>)) {
        self.members()
            .iter()
            .for_each(|val| val.visit_references(trace));
        self.class.visit_references(trace);
        self.super_.visit_references(trace);
    }

    fn size(&self) -> usize {
        core::mem::size_of::<Self>()
            + (self.class.nmembers as usize * core::mem::size_of::<Value>())
    }
}
