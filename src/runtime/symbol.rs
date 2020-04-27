use super::value::*;

#[repr(transparent)]
pub struct Symbol(pub Value);

impl Symbol {
    pub fn new_index(i: i32) -> Self {
        Self(Value::new_int(i as i32))
    }

    pub fn new_value(x: Value) -> Self {
        Self(x)
    }

    pub fn index(self) -> i32 {
        if self.0.is_any_int() {
            self.0.as_int32()
        } else {
            self.0.to_number().floor() as i32
        }
    }

    pub fn is_index(self) -> bool {
        self.0.is_number()
    }

    pub fn name(self) -> String {
        self.0.to_string()
    }

    pub fn dummy() -> Self {
        Self(Value::empty())
    }

    pub fn is_dummy(&self) -> bool {
        self.0.is_empty()
    }
}

use std::hash::{Hash, Hasher};

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.is_index() {
            self.index().hash(state);
        } else {
            self.name().hash(state);
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Symbol) -> bool {
        if self.is_index() && other.is_index() {
            self.index() == other.index()
        } else {
            self.name() == other.name()
        }
    }
}

impl Eq for Symbol {}

impl Copy for Symbol {}
impl Clone for Symbol {
    fn clone(&self) -> Self {
        *self
    }
}
