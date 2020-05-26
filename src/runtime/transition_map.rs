//! Hidden classes implementation based on https://mrale.ph/blog/2012/06/03/explaining-js-vms-in-js-inline-caches.html

use crate::*;
use gc::{Collectable, Handle};

#[derive(Copy, Clone)]
pub enum Descriptor {
    /// Property index
    Property(u32),
    /// Transition to class
    Transition(Handle<Class>),
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ClassKind {
    Slow,
    Fast,
}

impl Collectable for Class {
    fn walk_references(&self, trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        if let ClassKind::Fast = self.kind {
            for (key, descriptor) in self.descriptors.as_ref().unwrap().iter() {
                trace(key as *const Handle<String> as *const Handle<dyn Collectable>);
                match descriptor {
                    Descriptor::Transition(transition) => {
                        trace(transition as *const Handle<Class> as *const Handle<dyn Collectable>)
                    }
                    _ => (),
                }
            }

            for key in self.keys.as_ref().unwrap().iter() {
                trace(key as *const Handle<String> as *const Handle<dyn Collectable>);
            }
        }
    }
}

use indexmap::IndexMap;

pub struct Class {
    kind: ClassKind,
    descriptors: Option<Box<IndexMap<Handle<String>, Descriptor>>>,
    keys: Option<Box<Vec<Handle<String>>>>,
}

impl Class {
    pub fn new(kind: ClassKind) -> Self {
        Self {
            kind,
            descriptors: Some(Box::new(IndexMap::new())),
            keys: Some(Box::new(Vec::with_capacity(12))),
        }
    }
    pub fn add_property(&mut self, key: Handle<String>) -> Handle<Self> {
        let mut class = self.clone_class();
        class.get_mut().append(key);
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
            .find(|x| x.0.get() == key)
            .is_some()
    }

    pub fn get_descriptor(&self, key: &str) -> Option<&Descriptor> {
        self.descriptors.as_ref().unwrap().get(key)
    }
    pub fn append(&mut self, key: Handle<String>) {
        let id = self.keys.as_ref().unwrap().len();
        self.keys.as_mut().unwrap().push(key);
        self.descriptors
            .as_mut()
            .unwrap()
            .insert(key, Descriptor::Property(id as _));
    }

    pub fn clone_class(&self) -> Handle<Self> {
        let mut class = Class {
            kind: self.kind,
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
        get_rt().heap.allocate(class)
    }
}

use super::value::*;

pub enum Table {
    Fast(Handle<Class>, Vec<Value>),
    Slow(Handle<Class>, IndexMap<Handle<String>, Value>),
}

impl Table {
    pub fn class(&self) -> Handle<Class> {
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
    pub fn load(&self, key: &str) -> Option<Value> {
        match self {
            Table::Fast(cls, properties) => {
                let idx = self.find_property_for_read(key);
                if let Some(idx) = idx {
                    return Some(properties[idx as usize]);
                }
                None
            }
            Table::Slow(_, properties) => properties.get(key).copied(),
        }
    }

    pub fn set(&mut self, key: Handle<String>, value: Value) {
        match self {
            Table::Slow(_, properties) => match properties.entry(key) {
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
                        Table::Fast(_, properties) => {
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
    pub fn find_property_for_write(&mut self, key: Handle<String>) -> Option<u32> {
        match self {
            Table::Fast(class, properties) => {
                if !class.get().has_property(key.get()) {
                    if class.get().keys.as_ref().unwrap().len() > 12 {
                        return None;
                    }
                    *class = class.get_mut().add_property(key);
                    return match class.get().get_descriptor(key.get()) {
                        Some(Descriptor::Property(idx)) => Some(*idx),
                        _ => unreachable!(),
                    };
                }
                match class.get().get_descriptor(key.get()).copied() {
                    Some(Descriptor::Property(idx)) => Some(idx),
                    Some(Descriptor::Transition(new_class)) => {
                        *class = new_class;
                        return match class.get().get_descriptor(key.get()) {
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
    pub fn find_property_for_read(&self, key: impl AsRef<str>) -> Option<u32> {
        match self {
            Table::Fast(cls, ..) => {
                if let Some(Descriptor::Property(idx)) = cls.get().get_descriptor(key.as_ref()) {
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
            Table::Fast(class, properties) => {
                let mut map = IndexMap::new();
                for i in 0..class.get().keys.as_ref().unwrap().len() {
                    let key = class.get().keys.as_ref().unwrap()[i];
                    let val = properties[i];
                    map.insert(key, val);
                }
                class.get_mut().descriptors = None;
                class.get_mut().keys = None;
                class.get_mut().kind = ClassKind::Slow;
                *self = Table::Slow(*class, map);
            }
            _ => (),
        }
    }
}

impl Collectable for Table {
    fn walk_references(&self, trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        match self {
            Self::Fast(cls, props) => {
                cls.walk_references(trace);
                props.walk_references(trace);
            }
            Self::Slow(cls, props) => {
                cls.walk_references(trace);
                props.iter().for_each(|(key, val)| {
                    key.walk_references(trace);
                    val.walk_references(trace);
                });
            }
        }
    }
}
