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
pub extern "C" fn writeln(
    _: &RcState,
    _: &Arc<WaffleThread>,
    _: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    for value in arguments.iter() {
        print!("{}", value);
    }
    println!();
    Ok(ReturnValue::Value(Value::from(VTag::Null)))
}

pub extern "C" fn write(
    _: &RcState,
    _: &Arc<WaffleThread>,
    _: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    for value in arguments.iter() {
        print!("{}", value);
    }
    Ok(ReturnValue::Value(Value::from(VTag::Null)))
}

pub extern "C" fn readln(
    state: &RcState,
    proc: &Arc<WaffleThread>,
    _: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    let prompt = match arguments.len() {
        x if x == 0 => "".to_owned(),
        1 => arguments[0].to_string(),
        _ => "".to_owned(),
    };
    print!("{}", prompt);
    use std::io::{stdin, Read};
    let mut buf = String::new();
    match stdin().read_line(&mut buf) {
        Ok(_) => (),
        Err(e) => {
            return Err(Value::from(WaffleThread::allocate_string(
                proc,
                state,
                &e.to_string(),
            )))
        }
    }
    Ok(ReturnValue::Value(Value::from(
        WaffleThread::allocate_string(proc, state, &buf),
    )))
}

pub fn initialize_io(state: &RcState) {
    let io = Cell::with_prototype(CellValue::None, state.object_prototype.as_cell());
    let io = state.allocate(io);
    state.static_variables.lock().insert("io".to_owned(), io);
    let name = Arc::new("writeln".to_owned());
    let writeln = state.allocate_native_fn_with_name(writeln, "writeln", -1);
    io.as_cell()
        .add_attribute_without_barrier(&name, Value::from(writeln));

    let name = Arc::new("write".to_owned());
    let write = state.allocate_native_fn_with_name(write, "write", -1);
    io.as_cell()
        .add_attribute_without_barrier(&name, Value::from(write));
    io.as_cell().add_attribute_without_barrier(
        &Arc::new("readln".to_owned()),
        Value::from(state.allocate_native_fn(readln, "readln", -1)),
    );
}
