use super::value::*;
use std::collections::HashMap;
pub struct Module {
    pub name: Value,
    pub functions: HashMap<String, Value>,
    pub globals: HashMap<String, Value>,
}
