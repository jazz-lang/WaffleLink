use crate::runtime::value::Value;
pub struct Stack {
    values: Vec<Value>,
    stack: Vec<usize>,
}

impl Stack {
    fn increment(&mut self) {
        *self.stack.last_mut().unwrap() += 1;
    }
    fn decrement(&mut self) {
        let ix = self.stack.len() - 1;
        let i = self.stack[ix];
        self.stack[ix] = self.stack[ix].wrapping_sub(i);
    }
    pub fn push(&mut self, val: Value) {
        self.values.push(val);
        self.increment();
    }
    pub fn pop(&mut self) -> Option<Value> {
        self.values.pop().map(|elem| {
            self.decrement();
            elem
        })
    }

    pub fn enter(&mut self) {
        self.stack.push(0);
    }

    pub fn leave(&mut self) {
        self.stack.pop().map(|count| {
            let c = self.stack.len();
            self.stack.truncate(c - count);
            ()
        });
    }
}
