use super::value::*;
use crate::util::arc::Arc;
use std::vec::Vec;
pub struct Module {
    pub name: Arc<String>,
    pub globals: Vec<Value>,
    pub main_fn: Value,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Self {
            name: Arc::new(name.to_owned()),
            globals: vec![],
            main_fn: Value::empty(),
        }
    }
    pub fn get_global_at(&self, id: usize) -> Value {
        self.globals
            .get(id)
            .map(|x| *x)
            .unwrap_or(Value::from(VTag::Undefined))
    }
}
