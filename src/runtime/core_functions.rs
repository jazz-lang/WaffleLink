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
use super::*;
use crate::interpreter::context::*;
use crate::util::arc::Arc;
use std::sync::atomic::*;
pub extern "C" fn require(
    state: &RcState,
    process: &Arc<WaffleThread>,
    _: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    let rt: &Runtime = &RUNTIME;
    let mut registry = rt.registry.lock();
    let (module, not_loaded) = registry
        .load("", &arguments[0].to_string())
        .map_err(|err| Value::from(WaffleThread::allocate_string(process, state, &err)))?;
    drop(registry);
    if !not_loaded {
        return Ok(ReturnValue::Value(
            module.as_cell().module_value().unwrap().exports,
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
                parent: None,
                function: main_fn,
                code: function.code.clone(),
                module: function.module.clone(),
                registers: [Value::from(VTag::Undefined); 32],
                stack: vec![],
                this: Value::from(VTag::Undefined),
            };
            module.as_cell().module_value_mut().unwrap().exports = WaffleThread::allocate(
                process,
                Cell::with_prototype(CellValue::None, state.object_prototype.as_cell()),
            );
            process.push_context(ctx);
            let _ = RUNTIME.run(process)?;
            return Ok(ReturnValue::Value(
                module.as_cell().module_value().unwrap().exports,
            ));
        }
        _ => panic!("Function expected"),
    };
}

static LOADED: AtomicBool = AtomicBool::new(false);

pub extern "C" fn __start(
    state: &RcState,
    process: &Arc<WaffleThread>,
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
        state,
        process,
        Value::empty(),
        &[Value::from(
            state.intern(&format!("{}/Array.wfl", home_dir)),
        )],
    )?;
    log::trace!("Loading math builtins...");
    require(
        state,
        process,
        Value::empty(),
        &[Value::from(state.intern(&format!("{}/Math.wfl", home_dir)))],
    )?;
    log::trace!("Loadingg core builtins...");
    require(
        state,
        process,
        Value::empty(),
        &[Value::from(state.intern(&format!("{}/Core.wfl", home_dir)))],
    )?;
    log::debug!("--Builtins loaded--");
    Ok(ReturnValue::Value(Value::from(false)))
}

pub extern "C" fn instanceof(
    _: &RcState,
    _: &Arc<WaffleThread>,
    _: Value,
    args: &[Value],
) -> Result<ReturnValue, Value> {
    Ok(ReturnValue::Value(Value::from(args[0].is_kind_of(args[1]))))
}

pub extern "C" fn force_collect(
    _: &RcState,
    proc: &Arc<WaffleThread>,
    _: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    let rt: &Runtime = &RUNTIME;
    log::debug!("--Forced GC Cycle--");
    log::debug!("Suspending process...");
    Ok(ReturnValue::SuspendWaffleThread)
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
    lock.insert("require".to_owned(), Value::from(require));
    lock.insert("__start__".to_owned(), Value::from(start));
    lock.insert("instanceof".to_owned(), Value::from(instanceof));
    lock.insert("forceCollect".to_owned(), Value::from(force_collect));
    lock.insert("isNull".to_owned(), Value::from(to_bool));
}
