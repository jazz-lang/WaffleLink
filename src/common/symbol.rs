use crate::runtime::value::*;

#[derive(Copy, Clone)]
pub struct Symbol(Value);

impl Symbol {
    pub fn is_index(self) -> bool {
        self.0.is_number()
    }

    pub fn is_string(self) -> bool {
        !self.0.is_number()
    }

    pub fn get_index(self) -> i32 {
        self.0.as_int32()
    }
    pub fn get_string(self) -> String {
        self.0.to_string()
    }

    pub fn new(val: Value) -> Self {
        Self(val)
    }
}
