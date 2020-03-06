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

use super::basicblock::*;
use super::instruction::*;
use runtime::cell::*;
use crate::util::arc::Arc;
use crate::runtime;
pub mod cse;
pub mod load_after_store;
pub mod peephole;
pub mod ret_sink;
pub mod simplify;
pub mod simple_inlining;
pub mod tail_call_elim;

pub trait BytecodePass {
    fn execute(&mut self, f: &mut Arc<Function>);
}
