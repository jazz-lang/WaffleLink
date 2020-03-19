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
use super::process::*;
use super::scheduler::process_worker::ProcessWorker;
use super::state::*;
use super::value::*;
use super::*;
use crate::interpreter::context::*;
use crate::util::arc::Arc;

pub extern "C" fn load(
    _worker: &mut ProcessWorker,
    _state: &RcState,
    _process: &Arc<Process>,
    _: Value,
    _arguments: &[Value],
) -> Result<ReturnValue, Value> {
    unimplemented!()
}

pub extern "C" fn exports(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    this: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    let module = if this == state.module_prototype {
        process.context_ptr().module.clone()
    } else {
        this.as_cell().module_value().unwrap().clone()
    };

    Ok(ReturnValue::Value(module.exports))
}

pub fn initialize_module(state: &RcState) {
    let mut lock = state.static_variables.lock();
    state.module_prototype.add_attribute_without_barrier(
        state,
        Arc::new("exports".to_owned()),
        Value::from(state.allocate_native_fn(exports, "exports", 0)),
    );
    lock.insert("Module".to_owned(), state.module_prototype);
}
