/*
*   Copyright (c) 2020 Adel Prokurov
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/

use super::cell::*;
use super::value::*;
use std::collections::VecDeque;
pub struct Channel {
    pub messages: VecDeque<Value>,
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
