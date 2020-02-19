use super::cell::*;
use super::value::*;
use std::collections::VecDeque;
pub struct Channel {
    messages: VecDeque<Value>,
}
impl Channel {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::with_capacity(8),
        }
    }

    pub fn send(&mut self, value: Value) {
        self.messages.push_back(value)
    }

    pub fn receive(&mut self) -> Option<Value> {
        self.messages.pop_front()
    }

    pub fn trace<F>(&self, mut cb: F)
    where
        F: FnMut(*const CellPointer),
    {
        if self.has_messages() == false {
            return;
        }
        for value in self.messages.iter() {
            if value.is_cell() {
                unsafe { cb(&value.u.ptr) }
            }
        }
    }

    pub fn has_messages(&self) -> bool {
        !self.messages.is_empty()
    }
}
