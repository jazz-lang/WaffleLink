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
use std::fs::File;
use std::io::{Read, Write};
native_fn!(
    _worker,state,proc => read_only (path) {
        match File::open(path.to_string()) {
            Ok(file) => {
                let cell = Cell::with_prototype(CellValue::File(file), state.file_prototype.as_cell());
                Ok(ReturnValue::Value(Value::from(Process::allocate(proc, cell))))
            }
            Err(e) => Err(Value::from(Process::allocate_string(proc, state,&e.to_string())))
        }
    }
);

native_fn!(
    _worker,state,proc => read_bytes this (...args) {
        let size = match args.len() {
            2 => {
                let n = args[1].to_number();
                if n.is_nan() || n == std::f64::INFINITY || n == std::f64::NEG_INFINITY {
                    return Ok(ReturnValue::Value(Value::from(false)))
                }
                n.ceil() as usize
            },
            _ => return Ok(ReturnValue::Value(Value::from(false)))
        };
        let cell_buffer = if args[0].is_cell() {
            args[0].as_cell()
        } else {
            return Err(Value::from(Process::allocate_string(proc, state, "ByteArray or Array expected to File.read_bytes")));
        };
        let mut buffer = vec![0;size];
        let result = match this.as_cell().get_mut().value {
            CellValue::File(ref mut file) => {
                let file: &mut File = file;
                match file.take(size as _).read_to_end(&mut buffer) {
                    Ok(count) => count,
                    Err(e) => return Err(Value::from(Process::allocate_string(proc, state, &e.to_string())))
                }
            }
            _ => return Err(Value::from(Process::allocate_string(proc, state, "`this` is not an instance of File in File.read_bytes")))
        };
        match cell_buffer.get_mut().value {
            CellValue::Array(ref mut arr) => {
                arr.extend(buffer.iter().map(|x| Value::new_int(*x as _)))
            },
            CellValue::ByteArray(ref mut arr) => {
                arr.extend(buffer.into_iter())
            },
            _ => return Err(Value::from(Process::allocate_string(proc, state, "ByteArray or Array expected to File.read_bytes")))
        }
        Ok(ReturnValue::Value(Value::new_int(result as i32)))
    }
);

native_fn!(
    w,s,p => try_read_bytes this(...args) {
        match read_bytes(w,s,p,this,args) {
            Ok(v) => Ok(v),
            Err(_)=> Ok(ReturnValue::Value(Value::from(false)))
        }
    }
);

native_fn!(
    _w,state,proc => read_str this (..._args) {
        let result = match this.as_cell().get_mut().value {
            CellValue::File(ref mut file) => {
                let file: &mut File = file;
                let mut buf = String::new();
                file.read_to_string(&mut buf).map_err(|err| Value::from(Process::allocate_string(proc, state, &err.to_string())))?;
                buf
            }
            _ => return Err(Value::from(Process::allocate_string(proc, state, "`this` is not an instance of File in File.readToString")))
        };
        Ok(ReturnValue::Value(Value::from(Process::allocate_string(proc, state, &result))))
    }
);
pub fn initialize_file(state: &RcState) {
    let file = state.file_prototype.as_cell();
    let mut lock = state.static_variables.lock();
    file.add_attribute_without_barrier(
        &Arc::new("readOnly".to_owned()),
        Value::from(state.allocate_native_fn(read_only, "readOnly", -1)),
    );
    file.add_attribute_without_barrier(
        &Arc::new("readBytes".to_owned()),
        Value::from(state.allocate_native_fn(read_bytes, "readBytes", -1)),
    );
    file.add_attribute_without_barrier(
        &Arc::new("tryReadBytes".to_owned()),
        Value::from(state.allocate_native_fn(try_read_bytes, "tryReadBytes", -1)),
    );
    file.add_attribute_without_barrier(
        &Arc::new("readToString".to_owned()),
        Value::from(state.allocate_native_fn(read_str, "readToString", 0)),
    );
    lock.insert("File".to_owned(), Value::from(file));
}
