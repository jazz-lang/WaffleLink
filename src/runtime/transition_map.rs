use crate::*;
use gc::{Handle,Collectable};
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

impl Collectable for Class {
    fn walk_references(&self,trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        for (key,descriptor) in self.descriptors.iter() {
            trace(key as *const Handle<String> as *const Handle<dyn Collectable>);
            match descriptor {
                Descriptor::Transition(transition) => trace(transition as *const Handle<Class> as *const Handle<dyn Collectable>),
                _ => ()
            }
        }

        for key in self.keys.iter() {
            trace(key as *const Handle<String> as *const Handle<dyn Collectable>) ;
        }
    }
}

use indexmap::IndexMap;

pub struct Class {
    kind: ClassKind,
    descriptors: IndexMap<Handle<String>, Descriptor>,
    keys: Vec<Handle<String>>,
}

impl Class {
    pub fn add_property(&mut self, key: Handle<String>) -> Handle<Self> {
        let mut class = self.clone_class();
        class.get_mut().append(key);
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
        unimplemented!()
        //get_rt().heap.allocate(class).to_heap()
    }
}
