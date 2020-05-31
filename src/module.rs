use super::lock::Lock;
use super::value::*;
pub struct Module {
    pub lock: Lock,
    pub name: String,
    pub constants: Vec<Value>,
    pub entry: Value,
}

impl Module {}
