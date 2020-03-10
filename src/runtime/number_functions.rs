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
use super::state::*;
use super::threads::*;
use super::value::*;
use crate::util::arc::Arc;

native_fn!(
    _worker,state,proc => constructor (...args) {
        match args.len() {
            1 => {
                let string = args[0].to_string();
                if let Ok(value) = string.parse::<i32>() {
                    Ok(ReturnValue::Value(Value::new_int(value)))
                } else if let Ok(value) = string.parse::<f64>() {
                    Ok(ReturnValue::Value(Value::new_double(value)))
                } else {
                    Ok(ReturnValue::Value(Value::from(VTag::Null))) // TODO: Maybe it will be better to throw exception there?
                }
            }
            _ => Ok(ReturnValue::Value(Value::new_int(0))),
        }
    }
);

pub fn initialize_number(state: &RcState) {
    let mut lock = state.static_variables.lock();

    let number = state.number_prototype.as_cell();
    number.add_attribute_without_barrier(
        &Arc::new("constructor".to_owned()),
        Value::from(state.allocate_native_fn(constructor, "constructor", -1)),
    );
    lock.insert("Number".to_owned(), Value::from(number));
}
