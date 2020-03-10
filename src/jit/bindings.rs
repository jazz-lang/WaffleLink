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

use crate::interpreter;
use crate::runtime;
use crate::util::arc::Arc;
use interpreter::context::*;
use process::Process;
use runtime::cell::*;
use runtime::scheduler::process_worker::ProcessWorker;
use runtime::value::*;
use runtime::*;
#[no_mangle]
pub unsafe extern "C" fn value_slow_add(process: *const Arc<Process>, x: Value, y: Value) -> Value {
    let process = &*process;
    if x.is_bool() && y.is_bool() {
        return Value::new_int(x.as_bool() as i32 + y.as_bool() as i32);
    } else if x.is_null_or_undefined() && y.is_null_or_undefined() {
        return Value::from(VTag::Undefined);
    } else if x.is_cell() && y.is_cell() {
        let lhs = x.as_cell();
        let rhs = y.as_cell();
        if lhs.is_string() {
            return Value::from(Process::allocate_string(
                process,
                &RUNTIME.state,
                &format!("{}{}", lhs.to_string(), rhs.to_string()),
            ));
        } else {
            return Value::from(Process::allocate_string(
                process,
                &RUNTIME.state,
                &format!("{}{}", lhs.to_string(), rhs.to_string()),
            ));
        }
    } else {
        return Value::from(Process::allocate_string(
            process,
            &RUNTIME.state,
            &format!("{}{}", x.to_string(), y.to_string()),
        ));
    }
}

#[no_mangle]
pub unsafe extern "C" fn value_slow_gt(
    _process: *const Arc<Process>,
    lhs: Value,
    rhs: Value,
) -> Value {
    if lhs.is_bool() && rhs.is_bool() {
        return Value::from(lhs.as_bool() > rhs.as_bool());
    } else if lhs.is_cell() && rhs.is_cell() {
        let lhs = lhs.as_cell();
        let rhs = rhs.as_cell();
        if lhs.is_string() && rhs.is_string() {
            return Value::from(lhs.to_string().len() > rhs.to_string().len());
        } else if lhs.is_array() && rhs.is_array() {
            return Value::from(lhs.to_array().unwrap().len() > rhs.to_array().unwrap().len());
        } else {
            return Value::from(false);
        }
    } else {
        return Value::from(false);
    }
}

#[no_mangle]
pub unsafe extern "C" fn create_ret(val: Value) -> *const ReturnValue {
    Box::into_raw(Box::new(ReturnValue::Value(val)))
}

#[no_mangle]
pub unsafe extern "C" fn stack_pop(stack: *mut Vec<Value>) -> Value {
    let stack = &mut *stack;
    stack.pop().unwrap_or(Value::from(VTag::Undefined))
}

#[no_mangle]
pub unsafe extern "C" fn stack_push(stack: *mut Vec<Value>, v: Value) {
    (*stack).push(v);
}

#[no_mangle]
pub unsafe extern "C" fn value_call(
    proc: *mut Arc<Process>,
    worker: *mut ProcessWorker,
    stack: *mut Vec<Value>,
    function: Value,
    argc: u32,
) -> Value {
    let proc = &*proc;
    if !function.is_cell() {
        RUNTIME.run_default_panic(
            &*proc,
            &Process::allocate_string(
                proc,
                &RUNTIME.state,
                &format!("Cannot invoke '{}' value.", function.to_string()),
            ),
        );
        std::process::exit(1);
    }
    let cell = function.as_cell();
    let maybe_function = cell.function_value();
    let function: &Function = match maybe_function {
        Ok(function) => function,
        Err(_) => {
            RUNTIME.run_default_panic(
                &*proc,
                &Process::allocate_string(
                    proc,
                    &RUNTIME.state,
                    &format!("Cannot invoke '{}' value.", function.to_string()),
                ),
            );
            std::process::exit(1);
        }
    };
    let mut args = vec![];
    for _ in 0..argc {
        args.push(stack_pop(stack));
    }
    if let Some(native_fn) = function.native {
        let result = native_fn(
            &mut *worker,
            &RUNTIME.state,
            proc,
            Value::from(VTag::Undefined),
            &args,
        );
        if let Err(err) = result {
            panic!("{}", err.to_string());
            //throw!(self, process, err, context, index, bindex);
        }
        let result = result.unwrap();
        match result {
            ReturnValue::Value(value) => return value,
            ReturnValue::SuspendProcess => unimplemented!(),
            ReturnValue::YieldProcess => unimplemented!(),
        }
    } else {
        let mut new_context = Context::new();
        new_context.return_register = None;
        new_context.stack = args;
        new_context.function = Value::from(cell);
        new_context.module = function.module.clone();
        new_context.n = if proc.context_ptr().is_null() == false {
            proc.context_ptr().n
        } else {
            0
        };
        new_context.code = function.code.clone();
        new_context.terminate_upon_return = true;
        proc.push_context(new_context);
        RUNTIME.run(&mut *worker, proc).unwrap()
        //enter_context!(process, context, index, bindex);
    }
}

macro_rules! unimp {
    ($name: ident) => {
        #[no_mangle]
        pub extern "C" fn $name() {
            unimplemented!()
        }
    };
    ($($name: ident)*) => {
        $(
            unimp!($name);
        )*
    }
}
unimp!(value_to_double_slow value_slow_sub value_slow_div value_slow_mul value_slow_mod value_slow_lsh value_slow_rsh value_slow_eq value_slow_lt value_slow_lte value_slow_gte);
