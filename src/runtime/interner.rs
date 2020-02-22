use std::collections::HashMap;
use std::convert::AsRef;
use std::hash::{Hash, Hasher};

use super::cell::*;

#[derive(Clone, Copy)]
pub struct StringPointer {
    raw: *const String,
}

#[derive(Default)]
pub struct StringPool {
    mapping: HashMap<StringPointer, CellPointer>,
}

impl StringPointer {
    pub fn new(pointer: &String) -> Self {
        StringPointer {
            raw: pointer as *const String,
        }
    }
}

impl AsRef<String> for StringPointer {
    fn as_ref(&self) -> &String {
        unsafe { &*self.raw }
    }
}

unsafe impl Send for StringPointer {}
unsafe impl Sync for StringPointer {}

impl PartialEq for StringPointer {
    fn eq(&self, other: &StringPointer) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for StringPointer {}

impl Hash for StringPointer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl StringPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, string: &String) -> Option<CellPointer> {
        let pointer = StringPointer::new(string);

        self.mapping.get(&pointer).cloned()
    }

    /// Adds a new string to the string pool.
    ///
    /// This method will panic if the given ObjectPointer does not reside in the
    /// permanent space.
    pub fn add(&mut self, value: CellPointer) {
        if !value.is_permanent() {
            panic!("Only permanent objects can be stored in a string pool");
        }

        // Permanent pointers can not outlive a string pool, thus the below is
        // safe.
        let pointer = StringPointer::new(&value.to_string());

        self.mapping.insert(pointer, value);
    }
}
