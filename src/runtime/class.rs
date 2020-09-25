use super::cell_type::CellType;
use crate::prelude::*;
#[repr(C)]
pub struct Class {
    pub cell_type: CellType,
    pub name: Value,
    pub(crate) super_class: Option<Handle<Self>>,
    pub(crate) members: Handle<Map>,
}

impl Class {
    pub fn super_class(&self) -> Option<&Handle<Self>> {
        self.super_class.as_ref()
    }
    pub fn new(
        isolate: &RCIsolate,
        name: Value,
        fields: &[&str],
        super_class: Option<Handle<Self>>,
    ) -> Local<Self> {
        let map = Map::new(
            isolate,
            compute_hash_default,
            iseq_default,
            fields.len() as _,
        );
        let mut this = isolate.new_local(Self {
            cell_type: CellType::Class,
            name,
            super_class,
            members: map.to_heap(),
        });
        for field in fields {
            this.add_member(isolate, field);
        }
        this
    }
    /// Get offset of field in class.
    /// return: field offset
    pub fn member(&self, isolate: &RCIsolate, name: &str) -> Option<u32> {
        let interned = Value::new_sym(isolate.intern_str(name));
        self.members
            .lookup(isolate, interned)
            .map(|x| x.as_int32() as _)
    }
    /// Add field to class.
    /// return: field offset
    fn add_member(&mut self, isolate: &RCIsolate, name: &str) -> u32 {
        let interned = Value::new_sym(isolate.intern_str(name));
        if let Some(val) = self.members.lookup(isolate, interned) {
            return val.as_int32() as _;
        } else {
            let c = self.members.count();
            self.members
                .insert(isolate, interned, Value::new_int(c as _));
            c as u32
        }
    }
}

/// Instance of class. This is dynamically sized object, this means
/// each instance of this object might use different amount of memory
/// based on field count.
#[repr(C)]
pub struct Instance {
    pub cell_type: CellType,

    pub super_: Option<Handle<Self>>,
    pub class: Handle<Class>,
    members: Value,
}

impl Instance {
    pub fn new_self(isolate: &RCIsolate, class: Handle<Class>) -> Local<Self> {
        let mut this = isolate.new_local(Self {
            cell_type: CellType::Instance,
            super_: None,
            class,
            members: Value::undefined(),
        });
        this.members_mut()
            .iter_mut()
            .for_each(|x| *x = Value::undefined());
        this
    }

    pub fn new(isolate: &RCIsolate, class: Handle<Class>) -> Local<Self> {
        let mut obj = Self::new_self(isolate, class);
        let mut prev = obj.to_heap();
        let mut c = class.super_class;
        while let Some(x) = c {
            prev.super_ = Some(Self::new_self(isolate, x).to_heap());
            prev = prev.super_.unwrap();
            c = x.super_class;
        }
        obj
    }
    pub fn members(&self) -> &[Value] {
        unsafe { std::slice::from_raw_parts(&self.members, self.class.members.count()) }
    }

    pub fn members_mut(&mut self) -> &mut [Value] {
        unsafe { std::slice::from_raw_parts_mut(&mut self.members, self.class.members.count()) }
    }

    pub fn member(&self, isolate: &RCIsolate, name: &str) -> Option<Value> {
        if let Some(ix) = self.class.member(isolate, name) {
            return Some(self.members()[ix as usize]);
        } else {
            if let Some(super_) = self.super_ {
                super_.member(isolate, name)
            } else {
                None
            }
        }
    }
    pub fn member_mut(&mut self, isolate: &RCIsolate, name: &str) -> Option<&mut Value> {
        if let Some(ix) = self.class.member(isolate, name) {
            return Some(&mut self.members_mut()[ix as usize]);
        } else {
            if let Some(ref mut super_) = &mut self.super_ {
                super_.member_mut(isolate, name)
            } else {
                None
            }
        }
    }
}

impl Handle<Instance> {
    pub fn member_slot(&self, isolate: &RCIsolate, name: &str) -> Slot {
        if let Some(ix) = self.class.member(isolate, name) {
            return Slot {
                object: Some(*self),
                value: Some(self.members()[ix as usize]),
                ix,
            };
        } else {
            if let Some(super_) = self.super_ {
                super_.member_slot(isolate, name)
            } else {
                Slot::not_found()
            }
        }
    }
}

impl GcObject for Instance {
    fn visit_references(&self, tracer: &mut Tracer<'_>) {
        self.members()
            .iter()
            .for_each(|val| val.visit_references(tracer));
        self.class.visit_references(tracer);
        self.super_.visit_references(tracer);
    }

    fn size(&self) -> usize {
        core::mem::size_of::<Self>() + (self.class.members.count() * core::mem::size_of::<Value>())
    }
}

pub struct Slot {
    pub value: Option<Value>,
    pub object: Option<Handle<Instance>>,
    pub ix: u32,
}

impl Slot {
    pub fn found(&self) -> bool {
        self.ix != u32::MAX
    }
    pub fn not_found() -> Self {
        Self {
            value: None,
            object: None,
            ix: 0,
        }
    }
}

impl GcObject for Class {
    fn visit_references(&self, tracer: &mut Tracer<'_>) {
        self.name.visit_references(tracer);
        self.super_class.visit_references(tracer);
        self.members.visit_references(tracer);
    }
}
