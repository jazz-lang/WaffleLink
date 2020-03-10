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
use super::state::*;
use super::threads::*;
use super::value::*;
use crate::util::arc::Arc;
use regex::Regex;
pub extern "C" fn ctor(
    state: &RcState,
    process: &Arc<WaffleThread>,
    this: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    this.as_cell().get_mut().value = CellValue::Regex(Arc::new(
        Regex::new(&arguments[0].to_string())
            .map_err(|e| WaffleThread::allocate_string(process, state, &e.to_string()))?,
    ));
    Ok(ReturnValue::Value(this))
}

pub extern "C" fn is_match(
    state: &RcState,
    process: &Arc<WaffleThread>,
    this: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    match this.as_cell().get().value {
        CellValue::Regex(ref regex) => Ok(ReturnValue::Value(Value::from(
            regex.is_match(&arguments[0].to_string()),
        ))),
        _ => match type_error(
            state,
            process,
            Value::empty(),
            &[Value::from(WaffleThread::allocate_string(
                process,
                state,
                "Regex.isMatch called on null or undefined",
            ))],
        )? {
            ReturnValue::Value(val) => Err(val),
            _ => unreachable!(),
        },
    }
}

pub extern "C" fn find(
    state: &RcState,
    process: &Arc<WaffleThread>,
    this: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    match this.as_cell().get().value {
        CellValue::Regex(ref regex) => {
            let s = arguments[0].to_string();
            let match_ = regex.find(&s);
            if let Some(match_) = match_ {
                let mut match_object =
                    Cell::with_prototype(CellValue::None, state.object_prototype.as_cell());
                match_object.add_attribute(
                    Arc::new("start".to_owned()),
                    Value::new_int(match_.start() as _),
                );
                match_object.add_attribute(
                    Arc::new("end".to_owned()),
                    Value::new_int(match_.end() as _),
                );
                match_object.add_attribute(
                    Arc::new("str".to_owned()),
                    Value::from(Value::from(state.intern_string(match_.as_str().to_owned()))),
                );
                return Ok(ReturnValue::Value(WaffleThread::allocate(
                    process,
                    match_object,
                )));
            } else {
                return Ok(ReturnValue::Value(Value::from(VTag::Null)));
            }
        }
        _ => match type_error(
            state,
            process,
            Value::empty(),
            &[Value::from(WaffleThread::allocate_string(
                process,
                state,
                "Regex.isMatch called on null or undefined",
            ))],
        )? {
            ReturnValue::Value(val) => Err(val),
            _ => unreachable!(),
        },
    }
}

pub extern "C" fn find_iter(
    state: &RcState,
    process: &Arc<WaffleThread>,
    this: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    match this.as_cell().get().value {
        CellValue::Regex(ref regex) => {
            let s = arguments[0].to_string();
            let matches = regex.find_iter(&s);
            let mut array = vec![];
            for match_ in matches {
                let mut match_object =
                    Cell::with_prototype(CellValue::None, state.object_prototype.as_cell());
                match_object.add_attribute(
                    Arc::new("start".to_owned()),
                    Value::new_int(match_.start() as _),
                );
                match_object.add_attribute(
                    Arc::new("end".to_owned()),
                    Value::new_int(match_.end() as _),
                );
                match_object.add_attribute(
                    Arc::new("str".to_owned()),
                    Value::from(state.intern_string(match_.as_str().to_owned())),
                );
                array.push(Value::from(WaffleThread::allocate(process, match_object)));
            }

            Ok(ReturnValue::Value(Value::from(WaffleThread::allocate(
                process,
                Cell::with_prototype(
                    CellValue::Array(Box::new(array)),
                    state.array_prototype.as_cell(),
                ),
            ))))
        }
        _ => match type_error(
            state,
            process,
            Value::empty(),
            &[Value::from(WaffleThread::allocate_string(
                process,
                state,
                "Regex.isMatch called on null or undefined",
            ))],
        )? {
            ReturnValue::Value(val) => Err(val),
            _ => unreachable!(),
        },
    }
}

pub fn initialize_regex(state: &RcState) {
    let mut lock = state.static_variables.lock();

    let regex = state.allocate(Cell::with_prototype(
        CellValue::None,
        state.object_prototype.as_cell(),
    ));
    lock.insert("Regex".to_owned(), Value::from(regex));
    regex.add_attribute_without_barrier(
        state,
        Arc::new("constructor".to_owned()),
        Value::from(state.allocate_native_fn(ctor, "constructor", 1)),
    );
    regex.add_attribute_without_barrier(
        state,
        Arc::new("find".to_owned()),
        Value::from(state.allocate_native_fn(find, "find", 1)),
    );
    regex.add_attribute_without_barrier(
        state,
        Arc::new("findAll".to_owned()),
        Value::from(state.allocate_native_fn(find_iter, "findAll", 1)),
    );
}
