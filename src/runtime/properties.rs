use super::array::*;
use super::object::*;
use crate::gc::object::*;
use crate::values::*;
pub struct Properties {
    raw: Handle<Array>,
}

impl Properties {
    pub fn find_by_hash(&self, hash: u64) -> Option<Value> {
        for val in self.raw.as_slice() {
            if val.is_empty() {
                continue;
            } else {
                let cell = val.as_cell();
            }
        }
        None
    }
}
