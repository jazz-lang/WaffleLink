use crate::{object::*, value::*, *};
use indexmap::IndexMap;
#[derive(Copy, Clone)]
pub enum Descriptor {
    /// Property index
    Property(u32),
    /// Transition to class
    Transition(Ref<Class>),
}
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ClassKind {
    Slow,
    Fast,
}

#[repr(C)]
pub struct Class {
    header: object::Header,
    vtable: &'static vtable::VTable,
    kind: ClassKind,
    pub descriptors: Option<Box<IndexMap<Ref<WaffleString>, Descriptor>>>,
    pub keys: Option<Vec<Ref<WaffleString>>>,
}

pub static CLASS_VTBL: vtable::VTable = vtable::VTable {
    element_size: 0,
    instance_size: std::mem::size_of::<Class>(),
    parent: None,
    lookup_fn: None,
    index_fn: None,
    calc_size_fn: None,
    apply_fn: None,
    destroy_fn: None,
    trace_fn: Some(trace_class),
    set_fn: None,
    set_index_fn: None,
};

fn trace_class(this: Ref<Obj>, trace: &mut dyn FnMut(Ref<Obj>)) {
    let this = this.cast::<Class>();
    if let Some(descriptors) = &this.descriptors {
        for (key, desc) in descriptors.iter() {
            trace(key.cast());
            match desc {
                Descriptor::Transition(cls) => trace(cls.cast()),
                _ => (),
            }
        }
        if let Some(keys) = &this.keys {
            for i in 0..keys.len() {
                trace(keys[i].cast());
            }
        }
    }
}

impl Class {
    pub fn new(kind: ClassKind) -> Self {
        Self {
            header: object::Header::new(),
            vtable: &CLASS_VTBL,
            kind,
            descriptors: Some(Box::new(IndexMap::new())),
            keys: Some(Vec::with_capacity(12)),
        }
    }
    pub fn add_property(&mut self, key: Ref<WaffleString>) -> Ref<Self> {
        let mut class = self.clone_class();
        class.append(key);
        self.descriptors
            .as_mut()
            .unwrap()
            .insert(key, Descriptor::Transition(class));
        class
    }
    pub fn has_property(&self, key: &str) -> bool {
        self.descriptors
            .as_ref()
            .unwrap()
            .iter()
            .find(|x| x.0.str() == key)
            .is_some()
    }

    pub fn get_descriptor(&self, key: Ref<WaffleString>) -> Option<&Descriptor> {
        self.descriptors.as_ref().unwrap().get(&key)
    }
    pub fn append(&mut self, key: Ref<WaffleString>) {
        let id = self.keys.as_ref().unwrap().len();
        self.keys.as_mut().unwrap().push(key);
        self.descriptors
            .as_mut()
            .unwrap()
            .insert(key, Descriptor::Property(id as _));
    }

    pub fn clone_class(&self) -> Ref<Self> {
        let mut class = Class {
            kind: self.kind,
            vtable: self.vtable,
            header: Header::new(),
            keys: self.keys.clone(),
            descriptors: Some(Box::new(IndexMap::new())),
        };
        for i in 0..self.keys.as_ref().unwrap().len() {
            let key = self.keys.as_ref().unwrap()[i];
            class
                .descriptors
                .as_mut()
                .unwrap()
                .insert(key, self.descriptors.as_ref().unwrap()[&key]);
        }
        get_vm().allocate(class)
    }
}

#[repr(C)]
pub enum TableEnum {
    Fast(Ref<Class>, Vec<Value>),
    Slow(Ref<Class>, IndexMap<Ref<WaffleString>, Value>),
}

impl TableEnum {
    pub fn class(&self) -> Ref<Class> {
        match self {
            Self::Fast(cls, _) => *cls,
            Self::Slow(cls, _) => *cls,
        }
    }
    pub fn is_slow(&self) -> bool {
        match self {
            Self::Slow { .. } => true,
            _ => false,
        }
    }

    pub fn is_fast(&self) -> bool {
        !self.is_slow()
    }
    pub fn load(&self, key: Ref<WaffleString>) -> Option<Value> {
        match self {
            TableEnum::Fast(_cls, properties) => {
                let idx = self.find_property_for_read(key);
                if let Some(idx) = idx {
                    return Some(properties[idx as usize]);
                }
                None
            }
            TableEnum::Slow(_, properties) => properties.get(&key).copied(),
        }
    }

    pub fn set(&mut self, key: Ref<WaffleString>, value: Value) {
        match self {
            TableEnum::Slow(_, properties) => match properties.entry(key) {
                indexmap::map::Entry::Occupied(mut occupied) => {
                    occupied.insert(value);
                }
                indexmap::map::Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            },
            _ => {
                let idx = self.find_property_for_write(key);
                if let Some(idx) = idx {
                    match self {
                        TableEnum::Fast(_, properties) => {
                            properties[idx as usize] = value;
                            return;
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }

        self.convert_to_slow();
        self.set(key, value);
    }
    pub fn find_property_for_write(&mut self, key: Ref<WaffleString>) -> Option<u32> {
        match self {
            TableEnum::Fast(class, _properties) => {
                if !class.has_property(key.str()) {
                    if class.keys.as_ref().unwrap().len() > 12 {
                        return None;
                    }
                    
                    *class = class.add_property(key);
                    return match class.get_descriptor(key) {
                        Some(Descriptor::Property(idx)) => Some(*idx),
                        _ => unreachable!(),
                    };
                }
                match class.get_descriptor(key).copied() {
                    Some(Descriptor::Property(idx)) => Some(idx),
                    Some(Descriptor::Transition(new_class)) => {
                        *class = new_class;
                        return match class.get_descriptor(key) {
                            Some(Descriptor::Property(idx)) => Some(*idx),
                            _ => unreachable!(),
                        };
                    }
                    _ => unreachable!(),
                }
            }
            _ => None,
        }
    }
    pub fn find_property_for_read(&self, key: Ref<WaffleString>) -> Option<u32> {
        match self {
            TableEnum::Fast(cls, ..) => {
                if let Some(Descriptor::Property(idx)) = cls.get_descriptor(key) {
                    return Some(*idx);
                }
                return None;
            }
            _ => (),
        }
        None
    }
    pub fn convert_to_slow(&mut self) {
        match self {
            TableEnum::Fast(class, properties) => {
                let mut map = IndexMap::new();
                for i in 0..class.keys.as_ref().unwrap().len() {
                    let key = class.keys.as_ref().unwrap()[i];
                    let val = properties[i];
                    map.insert(key, val);
                }
                class.descriptors = None;
                class.keys = None;
                class.kind = ClassKind::Slow;
                *self = TableEnum::Slow(*class, map);
            }
            _ => (),
        }
    }
}

#[repr(C)]
pub struct Table {
    header: Header,
    vtable: &'static vtable::VTable,
    pub table: TableEnum,
}

impl Table {
    pub fn new(class: Option<Ref<Class>>) -> Self {
        let table = if let Some(cls) = class {
            if cls.kind == ClassKind::Slow {
                TableEnum::Slow(cls, IndexMap::new())
            } else {
                TableEnum::Fast(cls, vec![Value::undefined(); 12])
            }
        } else {
            TableEnum::Fast(
                get_vm().allocate(Class::new(ClassKind::Fast)),
                vec![Value::undefined(); 12],
            )
        };
        Self {
            header: Header::new(),
            vtable: &TABLE_VTBL,
            table: table,
        }
    }
    pub fn cls(&self) -> Ref<Class> {
        match self.table {
            TableEnum::Fast(cls, ..) => cls,
            TableEnum::Slow(cls, ..) => cls,
        }
    }
}

pub static TABLE_VTBL: vtable::VTable = vtable::VTable {
    element_size: 0,
    instance_size: std::mem::size_of::<Class>(),
    parent: None,
    lookup_fn: None,
    index_fn: None,
    calc_size_fn: None,
    apply_fn: None,
    destroy_fn: None,
    trace_fn: None,
    set_fn: None,
    set_index_fn: None,
};

fn trace_table(this: Ref<Obj>, trace: &mut dyn FnMut(Ref<Obj>)) {
    let this: Ref<Table> = this.cast();
    match &this.table {
        TableEnum::Fast(cls, props) => {
            trace(cls.cast());
            for prop in props.iter() {
                if prop.is_cell() && !prop.is_empty() {
                    trace(prop.as_cell());
                }
            }
        }
        TableEnum::Slow(cls, map) => {
            trace(cls.cast());
            for (prop, val) in map.iter() {
                trace(prop.cast());
                if val.is_cell() && !val.is_empty() {
                    trace(val.as_cell());
                }
            }
        }
    }
}
