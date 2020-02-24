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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i8)]
pub enum Phase {
    First = 0x00,
    Second,
    Third,
    Tracing,
    Fourth,
    Sweep,
}

impl Phase {
    #[inline]
    pub fn advance(&mut self) -> Phase {
        *self = unsafe { std::mem::transmute((*self as i8 + 1) % 6) };
        *self
    }
    #[inline]
    pub fn prev(&self) -> Self {
        unsafe { std::mem::transmute((*self as i8 - 1) % 6) }
    }

    #[inline]
    pub fn snooping(&self) -> bool {
        *self as i8 <= 1
    }

    #[inline]
    pub fn tracing(&self) -> bool {
        let val = *self as i8;
        val >= 1 && val <= 3
    }
}
