use super::*;
use gc::*;
use value::*;

/// Current thread state.
pub struct State {
    pub stack: CallStack,
}
impl State {
    pub fn new() -> Self {
        Self {
            stack: CallStack::new(),
        }
    }
}
impl Collectable for State {
    fn walk_references(&self, trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        /*match self.fiber {
            Some(ref f) => f.walk_references(trace),
            _ => (),
        }*/
    }
}

pub struct CallStack {
    pub stack: Vec<Frame>,
}

impl CallStack {
    pub fn new() -> Self {
        Self {
            stack: vec![],

        }
    }
    pub fn pop(&mut self) -> Option<Frame> {
        self.stack.pop().map(|frame| {
            frame
        })
    }


}

pub struct Frame {
    pub code: Handle<Vec<Vec<super::opcodes::Ins>>>,
    pub pc: super::opcodes::Pc,
    /// Our interpreter is reentrant so we have this flag.
    pub exit_on_return: bool,
    pub this: Value,
    pub env: Value,
    pub func: Value,
    pub acc: Value,
    pub stack: Vec<Value>,
    used: usize,
}
