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
pub extern "C" fn array_new(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    this: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    let array = Box::new(arguments.to_vec());
    if this.is_cell() {
        this.as_cell().get_mut().value = CellValue::Array(array);
        this.as_cell()
            .set_prototype(state.array_prototype.as_cell());
        return Ok(ReturnValue::Value(this));
    } else {
        let value = Process::allocate(
            process,
            Cell::with_prototype(CellValue::Array(array), state.array_prototype.as_cell()),
        );
        return Ok(ReturnValue::Value(value.into()));
    }
}

pub extern "C" fn array_pop(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    this: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    if !this.is_cell() {
        return Err(Value::from(Process::allocate_string(
            process,
            state,
            "not an array",
        )));
    }
    let cell = this.as_cell();
    if cell.is_array() {
        Ok(ReturnValue::Value(
            cell.array_value_mut()
                .unwrap()
                .pop()
                .unwrap_or(Value::from(VTag::Undefined)),
        ))
    } else {
        return Err(Value::from(Process::allocate_string(
            process,
            state,
            "not an array",
        )));
    }
}

pub extern "C" fn array_push(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    this: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    if !this.is_cell() {
        return Err(Value::from(Process::allocate_string(
            process,
            state,
            "not an array",
        )));
    }
    let cell = this.as_cell();
    if cell.is_array() {
        cell.array_value_mut().unwrap().push(
            arguments
                .get(0)
                .copied()
                .unwrap_or(Value::from(VTag::Undefined)),
        );
        Ok(ReturnValue::Value(Value::from(VTag::Null)))
    } else {
        return Err(Value::from(Process::allocate_string(
            process,
            state,
            "not an array",
        )));
    }
}

pub extern "C" fn array_length(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    this: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    if !this.is_cell() {
        return Err(Value::from(Process::allocate_string(
            process,
            state,
            "not an array",
        )));
    }
    let cell = this.as_cell();
    if cell.is_array() {
        Ok(ReturnValue::Value(Value::new_int(
            cell.array_value().unwrap().len() as _,
        )))
    } else {
        return Err(Value::from(Process::allocate_string(
            process,
            state,
            "not an array",
        )));
    }
}

pub extern "C" fn array_remove(
    _: &mut ProcessWorker,
    _state: &RcState,
    _process: &Arc<Process>,
    this: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    let index = arguments[0].to_number();
    if index.is_infinite() || index.is_nan() {
        return Ok(ReturnValue::Value(Value::from(VTag::Undefined)));
    }
    let index = index.ceil() as i64 as usize;

    let val = match this.as_cell().get_mut().value {
        CellValue::Array(ref mut arr) if index < arr.len() => arr.remove(index),
        _ => return Ok(ReturnValue::Value(Value::from(VTag::Undefined))),
    };
    return Ok(ReturnValue::Value(val));
}

pub fn initialize_array_builtins(state: &RcState) {
    let array_prototype: CellPointer = state.array_prototype.as_cell();
    let new = state.allocate_native_fn(array_new, "constructor", -1);
    array_prototype
        .add_attribute_without_barrier(&Arc::new("constructor".to_owned()), Value::from(new));
    let length = state.allocate_native_fn(array_length, "length", 0);
    array_prototype
        .add_attribute_without_barrier(&Arc::new("length".to_owned()), Value::from(length));
    let push = state.allocate_native_fn(array_push, "push", 1);
    array_prototype.add_attribute_without_barrier(&Arc::new("push".to_owned()), Value::from(push));
    let pop = state.allocate_native_fn(array_pop, "pop", 0);
    array_prototype.add_attribute_without_barrier(&Arc::new("pop".to_owned()), Value::from(pop));
    array_prototype.add_attribute_without_barrier(
        &Arc::new("remove".to_owned()),
        Value::from(state.allocate_native_fn(array_remove, "remove", 1)),
    );
    state
        .static_variables
        .lock()
        .insert("Array".to_owned(), state.array_prototype);
}
