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
use crate::util::arc::Arc;

pub extern "C" fn type_error(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    _: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    let msg = if let Some(arg) = arguments.get(0) {
        format!("TypeError: {}", arg.to_string())
    } else {
        format!("TypeError")
    };
    let proto = state
        .static_variables
        .lock()
        .get("TypeError")
        .copied()
        .unwrap()
        .as_cell();
    let mut cell = Cell::with_prototype(CellValue::None, proto);
    cell.add_attribute(
        Process::allocate_string(process, state, "message").into(),
        Process::allocate_string(process, state, &msg).into(),
    );
    Ok(ReturnValue::Value(Value::from(Process::allocate(
        process, cell,
    ))))
}

pub extern "C" fn exception_to_string(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    this: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    let message = this.lookup_attribute(
        state,
        &Value::from(Process::allocate_string(process, state, "message")),
    );
    let message = if let None = message {
        "Unknown exception".to_owned()
    } else if let Some(message) = message {
        message.to_string()
    } else {
        unreachable!()
    };

    Ok(ReturnValue::Value(Value::from(Process::allocate_string(
        process, state, &message,
    ))))
}

pub fn initialize_exception(state: &RcState) {
    let mut vars = state.static_variables.lock();
    let exception = state.allocate(Cell::with_prototype(
        CellValue::None,
        state.object_prototype.as_cell(),
    ));
    exception.add_attribute_without_barrier(
        state,
        Value::from(state.intern_string("toString".to_owned())),
        Value::from(state.allocate_native_fn(exception_to_string, "toString", 0)),
    );
    vars.insert("Exception".to_owned(), Value::from(exception));
    let cell = state.allocate(Cell::with_prototype(CellValue::None, exception.as_cell()));
    cell.add_attribute_without_barrier(
        state,
        Value::from(state.intern_string("constructor".to_owned())),
        Value::from(state.allocate_native_fn(type_error, "constructor", -1)),
    );

    vars.insert("TypeError".to_owned(), Value::from(cell));
}
