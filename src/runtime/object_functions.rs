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

pub extern "C" fn constructor(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    this: Value,
    args: &[Value],
) -> Result<ReturnValue, Value> {
    let prototype = match args.len() {
        1 => {
            if args[0].is_cell() {
                args[0].as_cell()
            } else {
                state.object_prototype.as_cell()
            }
        }
        _ => state.object_prototype.as_cell(),
    };
    if !this.is_cell() {
        let object = Process::allocate(process, Cell::with_prototype(CellValue::None, prototype));

        return Ok(ReturnValue::Value(Value::from(object)));
    } else {
        let cell = this.as_cell();
        cell.get_mut().set_prototype(prototype);
        cell.get_mut().value = CellValue::None;
        return Ok(ReturnValue::Value(this));
    }
}

pub extern "C" fn to_string(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    this: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    Ok(ReturnValue::Value(Value::from(Process::allocate_string(
        process,
        state,
        &this.to_string(),
    ))))
}

pub extern "C" fn keys(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    _: Value,
    args: &[Value],
) -> Result<ReturnValue, Value> {
    let object = args[0].as_cell();
    let mut strings = vec![];
    for key in object.get().attribute_names() {
        strings.push(key);
    }

    let array = strings
        .iter()
        .map(|x| Value::from(x.as_cell()))
        .collect::<Vec<_>>();
    Ok(ReturnValue::Value(Value::from(Process::allocate(
        process,
        Cell::with_prototype(
            CellValue::Array(Box::new(array)),
            state.array_prototype.as_cell(),
        ),
    ))))
}

pub fn initialize_object(state: &RcState) {
    let mut lock = state.static_variables.lock();
    let object = state.object_prototype.as_cell();
    object.add_attribute_without_barrier(
        &Value::from(state.intern_string("constructor".to_owned())),
        Value::from(state.allocate_native_fn(constructor, "constructor", -1)),
    );
    object.add_attribute_without_barrier(
        &Value::from(state.intern_string("toString".to_owned())),
        Value::from(state.allocate_native_fn(to_string, "toString", 0)),
    );
    object.add_attribute_without_barrier(
        &Value::from(state.intern_string("keys".to_owned())),
        Value::from(state.allocate_native_fn(keys, "keys", 1)),
    );
    lock.insert("Object".to_owned(), Value::from(object));
}
