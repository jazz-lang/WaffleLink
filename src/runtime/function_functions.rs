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
use crate::interpreter::context::*;
use crate::util::arc::Arc;

pub fn invoke_value(
    worker: &mut ProcessWorker,
    proc: &Arc<Process>,
    state: &RcState,
    value: Value,
    this: Value,
    args: Value,
) -> Result<ReturnValue, Value> {
    let cell = value.as_cell();
    let args = if args.is_cell() {
        match args.as_cell().get().value {
            CellValue::Array(ref array) => array.clone().to_vec(),
            _ => vec![],
        }
    } else {
        vec![]
    };
    match cell.get().value {
        CellValue::Function(ref func) => {
            if let Some(func) = func.native {
                return func(worker, state, proc, this, &args);
            } else {
                let mut ctx = Context::new();
                ctx.n = proc.context_ptr().n + 1;
                ctx.module = func.module.clone();
                ctx.code = func.code.clone();
                ctx.arguments = args.clone();
                ctx.function = Value::from(cell);
                ctx.terminate_upon_return = true;
                ctx.stack = args;
                ctx.this = this;
                proc.push_context(ctx);
                let result = RUNTIME.run(worker, proc);
                return result.map(|x| ReturnValue::Value(x));
            }
        }
        _ => unimplemented!(),
    }
}

native_fn!(
    _worker,state,proc => bytecode this(..._args) {
        let this = this.as_cell();
        match this.get().value {
            CellValue::Function(ref func) => {
                let mut array = vec![];
                for bb in func.code.iter() {
                    let mut cell = Cell::with_prototype(CellValue::None,state.object_prototype.as_cell());
                    cell.add_attribute(Process::allocate_string(proc, state,"index").into(),Value::new_int(bb.index as _));
                    let mut ins_array = vec![];
                    for ins in bb.instructions.iter() {
                        let string = format!("{:?}",ins);
                        let mut split = string.split('(');
                        let name = split.next().unwrap();
                        let args = ins.args().iter().map(|x| if *x < std::u32::MAX as u64 {Value::new_int(*x as _)} else {Value::new_double(f64::from_bits(*x))}).collect();
                        let mut ins_cell = Cell::with_prototype(CellValue::None,state.object_prototype.as_cell());
                        ins_cell.add_attribute(Process::allocate_string(proc, state,"name").into(),Value::from(state.intern_string(name.to_owned())));
                        ins_cell.add_attribute(Process::allocate_string(proc, state,"operands").into(),Value::from(Process::allocate(proc,Cell::with_prototype(CellValue::Array(Box::new(args)),state.array_prototype.as_cell()))));
                        ins_array.push(Value::from(Process::allocate(proc,ins_cell)));
                    }

                    let array_ = Value::from(Process::allocate(proc,Cell::with_prototype(CellValue::Array(Box::new(ins_array)),state.array_prototype.as_cell())));
                    cell.add_attribute(Process::allocate_string(proc, state, "instructions").into(),array_);
                    array.push(Value::from(Process::allocate(proc,cell)));
                }

                let ret = Process::allocate(proc,Cell::with_prototype(CellValue::Array(Box::new(array)),state.array_prototype.as_cell()));
                Ok(ReturnValue::Value(Value::from(ret)))
            }
            _ => return Ok(ReturnValue::Value(Value::from(VTag::Null)))
        }
    }
);
native_fn!(
    _worker,state,proc => apply this (...args) {
        invoke_value(_worker,proc,state,this,args[0],args[1])
    }
);

native_fn!(
    _worker,state,proc => arguments this(..._args) {
        let args = proc.context_ptr().arguments.clone();
        let cell = Process::allocate(proc,Cell::with_prototype(CellValue::Array(Box::new(args)),state.array_prototype.as_cell()));
        Ok(ReturnValue::Value(Value::from(cell)))
    }
);

pub fn initialize_function(state: &RcState) {
    let mut lock = state.static_variables.try_lock().unwrap();
    let func = state.function_prototype.as_cell();
    func.add_attribute_without_barrier(
        &Value::from(state.intern_string("apply".to_owned())),
        Value::from(state.allocate_native_fn(apply, "apply", 2)),
    );
    func.add_attribute_without_barrier(
        &Value::from(state.intern_string("bytecode".to_owned())),
        Value::from(state.allocate_native_fn(bytecode, "bytecode", 0)),
    );
    func.add_attribute_without_barrier(
        &Value::from(state.intern_string("arguments".to_owned())),
        Value::from(state.allocate_native_fn(arguments, "arguments", 0)),
    );
    lock.insert("Function".to_owned(), Value::from(func));
}
