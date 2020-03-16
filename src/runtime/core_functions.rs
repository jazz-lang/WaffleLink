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
use crate::heap::gc_pool::Collection;
use crate::interpreter::context::*;
use crate::util::arc::Arc;
use std::sync::atomic::*;
pub extern "C" fn require(
    worker: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    _: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    let rt: &Runtime = &RUNTIME;
    let mut registry = rt.registry.lock();
    let (module, not_loaded) = registry
        .load("", &arguments[0].to_string())
        .map_err(|err| Value::from(Process::allocate_string(process, state, &err)))?;
    drop(registry);
    if !not_loaded {
        return Ok(ReturnValue::Value(
            process
                .local_data_mut()
                .heap
                .copy_object(process, module.as_cell().module_value().unwrap().exports),
        ));
    }
    let main_fn = module.as_cell().module_value().unwrap().main_fn;
    let function = main_fn.as_cell();
    match function.get().value {
        CellValue::Function(ref function) => {
            let ctx = Context {
                arguments: vec![],
                terminate_upon_return: true,
                return_register: None,
                n: process.context_ptr().n,
                in_tail: false,
                index: 0,
                bindex: 0,
                generator: None,
                parent: None,
                function: main_fn,
                code: function.code.clone(),
                module: function.module.clone(),
                registers: [Value::from(VTag::Undefined); 32],
                stack: vec![],
                this: Value::from(VTag::Undefined),
            };
            module.as_cell().module_value_mut().unwrap().exports = Process::allocate(
                process,
                Cell::with_prototype(CellValue::None, state.object_prototype.as_cell()),
            )
            .into();
            process.push_context(ctx);
            let _ = RUNTIME.run(worker, process)?;
            return Ok(ReturnValue::Value(
                module.as_cell().module_value().unwrap().exports,
            ));
        }
        _ => panic!("Function expected"),
    };
}

static LOADED: AtomicBool = AtomicBool::new(false);

pub extern "C" fn __start(
    worker: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    _: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    if LOADED.load(Ordering::Acquire) {
        return Ok(ReturnValue::Value(Value::from(true)));
        //return Err(Value::from(state.intern_string(String::from("already loaded"))));
    }
    let home_dir = format!(
        "{}/.waffle/builtins/",
        dirs::home_dir().unwrap().to_str().unwrap().to_owned()
    );
    log::debug!("--Loading builtins--");
    LOADED.store(true, Ordering::Release);
    log::trace!("Loading array builtins...");
    require(
        worker,
        state,
        process,
        Value::empty(),
        &[Value::from(
            state.intern(&format!("{}/Array.wfl", home_dir)),
        )],
    )?;
    log::trace!("Loading math builtins...");
    require(
        worker,
        state,
        process,
        Value::empty(),
        &[Value::from(state.intern(&format!("{}/Math.wfl", home_dir)))],
    )?;
    log::trace!("Loadingg core builtins...");
    require(
        worker,
        state,
        process,
        Value::empty(),
        &[Value::from(state.intern(&format!("{}/Core.wfl", home_dir)))],
    )?;
    log::debug!("--Builtins loaded--");
    Ok(ReturnValue::Value(Value::from(false)))
}

pub extern "C" fn instanceof(
    _: &mut ProcessWorker,
    _: &RcState,
    _: &Arc<Process>,
    _: Value,
    args: &[Value],
) -> Result<ReturnValue, Value> {
    Ok(ReturnValue::Value(Value::from(args[0].is_kind_of(args[1]))))
}

pub extern "C" fn type_of(
    _: &mut ProcessWorker,
    state: &RcState,
    proc: &Arc<Process>,
    _: Value,
    args: &[Value],
) -> Result<ReturnValue, Value> {
    let value = args[0];
    if value.is_null_or_undefined() {
        return Ok(ReturnValue::Value(Value::from(Process::allocate_string(
            proc, state, "null",
        ))));
    } else if value.is_number() {
        return Ok(ReturnValue::Value(Value::from(Process::allocate_string(
            proc, state, "number",
        ))));
    } else if value.is_bool() {
        return Ok(ReturnValue::Value(Value::from(Process::allocate_string(
            proc, state, "bool",
        ))));
    } else {
        match value.as_cell().get().value {
            CellValue::Array(_) => {
                return Ok(ReturnValue::Value(Value::from(Process::allocate_string(
                    proc, state, "array",
                ))))
            }
            CellValue::String(_) | CellValue::InternedString(_) => {
                return Ok(ReturnValue::Value(Value::from(Process::allocate_string(
                    proc, state, "string",
                ))))
            }
            CellValue::Regex(_) => {
                return Ok(ReturnValue::Value(Value::from(Process::allocate_string(
                    proc, state, "regex",
                ))))
            }
            CellValue::File(_) => {
                return Ok(ReturnValue::Value(Value::from(Process::allocate_string(
                    proc, state, "file",
                ))))
            }
            CellValue::Duration(_) => {
                return Ok(ReturnValue::Value(Value::from(Process::allocate_string(
                    proc, state, "duration",
                ))))
            }
            CellValue::Function(_) => {
                return Ok(ReturnValue::Value(Value::from(Process::allocate_string(
                    proc, state, "function",
                ))))
            }
            CellValue::Module(_) => {
                return Ok(ReturnValue::Value(Value::from(Process::allocate_string(
                    proc, state, "module",
                ))))
            }
            CellValue::ByteArray(_) => {
                return Ok(ReturnValue::Value(Value::from(Process::allocate_string(
                    proc,
                    state,
                    "bytearray",
                ))))
            }
            _ => {
                return Ok(ReturnValue::Value(Value::from(Process::allocate_string(
                    proc, state, "object",
                ))))
            }
        }
    }
}

pub extern "C" fn force_collect(
    _: &mut ProcessWorker,
    _: &RcState,
    proc: &Arc<Process>,
    _: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    let rt: &Runtime = &RUNTIME;
    log::debug!("--Forced GC Cycle--");
    rt.state.gc_pool.schedule(Collection::new(proc.clone()));
    log::debug!("Suspending process...");
    Ok(ReturnValue::SuspendProcess)
}

native_fn!(
    _worker,_state,_proc => is_null_or_undefined(arg) {
        Ok(ReturnValue::Value(Value::from(arg.is_null_or_undefined())))
    }
);

pub fn initialize_core(state: &RcState) {
    let mut lock = state.static_variables.lock();
    let require = state.allocate_native_fn(require, "require", 1);
    let to_bool = state.allocate_native_fn(is_null_or_undefined, "isNull", 1);
    let start = state.allocate_native_fn(__start, "__start__", 0);
    let instanceof = state.allocate_native_fn(instanceof, "instanceof", 2);
    let force_collect = state.allocate_native_fn(force_collect, "forceCollect", 0);
    let tyof = state.allocate_native_fn(type_of, "typeof", 1);
    lock.insert("typeof".to_owned(), Value::from(tyof));
    lock.insert("require".to_owned(), Value::from(require));
    lock.insert("__start__".to_owned(), Value::from(start));
    lock.insert("instanceof".to_owned(), Value::from(instanceof));
    lock.insert("forceCollect".to_owned(), Value::from(force_collect));
    lock.insert("isNull".to_owned(), Value::from(to_bool));
}
