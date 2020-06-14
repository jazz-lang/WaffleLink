use crate::value::Value;

pub struct Module {
    pub name: Value,
    pub constants: Vec<Value>,
}
