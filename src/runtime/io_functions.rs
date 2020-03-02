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
pub extern "C" fn writeln(
    _: &mut ProcessWorker,
    _: &RcState,
    _: &Arc<Process>,
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
    _: &mut ProcessWorker,
    _: &RcState,
    _: &Arc<Process>,
    _: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    for value in arguments.iter() {
        print!("{}", value);
    }
    Ok(ReturnValue::Value(Value::from(VTag::Null)))
}

pub fn initialize_io(state: &RcState) {
    let io = Cell::with_prototype(CellValue::None, state.object_prototype.as_cell());
    let io = state.allocate(io);
    state.static_variables.lock().insert("IO".to_owned(), io);
    let name = Arc::new("WriteLine".to_owned());
    let writeln = state.allocate_native_fn_with_name(writeln, "WriteLine", -1);
    io.as_cell()
        .add_attribute_without_barrier(&name, Value::from(writeln));

    let name = Arc::new("Write".to_owned());
    let write = state.allocate_native_fn_with_name(write, "Write", -1);
    io.as_cell()
        .add_attribute_without_barrier(&name, Value::from(write));
}
