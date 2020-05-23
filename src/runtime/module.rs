use super::cell::CellPointer;
use super::value::Value;
pub struct Module {
    pub name: Value,
    pub constants: Vec<Value>,
    pub main: Value,
}

impl Module {
    pub fn new(name: Value) -> Self {
        Self {
            name,
            constants: vec![],
            main: Value::undefined(),
        }
    }

    pub fn add_constant(&mut self, x: Value) -> usize {
        let p = self.constants.len();
        self.constants.push(x);
        p
    }

    pub fn each_pointer(&mut self, stack: &mut std::collections::VecDeque<*const CellPointer>) {
        self.name.each_pointer(stack);
        self.constants.iter().for_each(|x| x.each_pointer(stack));
    }
}
