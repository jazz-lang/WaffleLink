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
use super::exception::*;
use super::process::*;
use super::scheduler::process_worker::*;
use super::state::*;
use super::value::*;
use crate::util::arc::Arc;

pub extern "C" fn get_home(
    _: &mut ProcessWorker,
    state: &RcState,
    _process: &Arc<Process>,
    _: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    Ok(ReturnValue::Value(Value::from(state.intern_string(
        dirs::home_dir().unwrap().to_str().unwrap().to_owned(),
    ))))
}

pub extern "C" fn arguments(
    _: &mut ProcessWorker,
    state: &RcState,
    proc: &Arc<Process>,
    _: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    let args = std::env::args()
        .map(|x| Process::allocate_string(proc, state, &x))
        .map(Value::from)
        .collect::<Vec<_>>();
    Ok(ReturnValue::Value(Value::from(Process::allocate(
        proc,
        Cell::with_prototype(
            CellValue::Array(Box::new(args)),
            state.array_prototype.as_cell(),
        ),
    ))))
}

pub fn initialize_env(state: &RcState) {
    let mut lock = state.static_variables.lock();
    let mut env = Cell::with_prototype(CellValue::None, state.object_prototype.as_cell());
    env.add_attribute(
        Value::from(state.intern_string("getHome".to_owned())),
        Value::from(state.allocate_native_fn(get_home, "getHome", 0)),
    );
    env.add_attribute(
        Value::from(state.intern_string("arguments".to_owned())),
        Value::from(state.allocate_native_fn(arguments, "arguments", 0)),
    );
    lock.insert("env".to_owned(), Value::from(state.allocate(env)));
}
