use super::*;
use fiber::*;
use gc::*;
use value::*;
/// Current thread state.
pub struct State {
    pub fiber: Option<Handle<Fiber>>,
}

impl Collectable for State {
    fn walk_references(&self, trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        match self.fiber {
            Some(ref f) => f.walk_references(trace),
            _ => (),
        }
    }
}

pub struct CallStack {
    pub stack: Vec<Frame>,
    pub value_stack: Vec<Value>,
}

impl CallStack {
    pub fn pop(&mut self) -> Option<Frame> {
        self.stack.pop().map(|frame| {
            // Clear stack
            for _ in 0..frame.used {
                self.value_stack.pop().unwrap(); // stack *must* have some values.
            }
            frame
        })
    }

    pub fn push_value(&mut self, val: Value) {
        debug_assert!(!self.stack.is_empty());
        self.value_stack.push(val);
        self.stack.last_mut().unwrap().used += 1;
    }

    pub fn pop_value(&mut self) -> Option<Value> {
        debug_assert!(!self.stack.is_empty());
        let v = self.value_stack.pop();
        self.stack.last_mut().unwrap().used -= 1;
        v
    }
}

pub struct Frame {
    pub code: Handle<Vec<u8>>,
    pub ip: usize,
    /// Our interpreter is reentrant so we have this flag.
    pub exit_on_return: bool,
    pub this: Value,
    pub env: Value,
    pub func: Value,
    pub acc: Value,
    used: usize,
}
