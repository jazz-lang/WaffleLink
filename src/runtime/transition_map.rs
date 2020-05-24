use crate::*;
use cgc::api::*;
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Transition;
#[derive(Copy, Clone)]
pub enum Descriptor {
    Property(u32),
    Transition(Handle<Class>),
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ClassKind {
    Slow,
    Fast,
}

use indexmap::IndexMap;

pub struct Class {
    kind: ClassKind,
    descriptors: IndexMap<Handle<String>, Descriptor>,
    keys: Vec<Handle<String>>,
}

impl Traceable for Class {
    fn trace_with(&self, tracer: &mut Tracer) {
        for (key, desc) in self.descriptors.iter() {
            tracer.trace(key);
            match desc {
                Descriptor::Transition(ref c) => {
                    tracer.trace(c);
                }
                _ => (),
            }
        }
        for key in self.keys.iter() {
            tracer.trace(key);
        }
    }
}

impl Finalizer for Class {}
impl Class {
    pub fn add_property(&mut self, key: Handle<String>) -> Handle<Self> {
        let mut class = self.clone_class();
        class.append(key);
        self.descriptors.insert(key, Descriptor::Transition(class));
        class
    }

    pub fn append(&mut self, key: Handle<String>) {
        let id = self.keys.len();
        self.keys.push(key);
        self.descriptors.insert(key, Descriptor::Property(id as _));
    }

    pub fn clone_class(&self) -> Handle<Self> {
        let mut class = Class {
            kind: self.kind,
            keys: self.keys.clone(),
            descriptors: IndexMap::new(),
        };
        for i in 0..self.keys.len() {
            let key = self.keys[i];
            class.descriptors.insert(key, self.descriptors[&key]);
        }
        get_rt().heap.allocate(class).to_heap()
    }
}
