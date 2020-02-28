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

pub mod context;
pub mod tracing_interpreter;
use crate::bytecode::instruction::*;
use crate::heap::*;
use crate::runtime::*;
use crate::util::arc::Arc;
use crate::util::ptr::Ptr;
use cell::*;
use context::*;
use process::*;
use scheduler::process_worker::ProcessWorker;
use value::*;

macro_rules! reset_context {
    ($process:expr, $context:ident, $index:ident,$bindex: ident) => {{
        assert!(!$process.context_ptr().raw.is_null());
        $context = $process.context_ptr();
        $index = $context.index;
        $bindex = $context.bindex;
    }};
}

macro_rules! remember_and_reset {
    ($process: expr, $context: ident, $index: ident,$bindex: ident) => {
        $context.index = $index - 1;

        reset_context!($process, $context, $index, $bindex);
        continue;
    };
}

macro_rules! throw {
    ($rt: expr,$proc: expr,$value: expr,$context: ident,$index: ident, $bindex: ident) => {
        $context.index = $index;
        $context.bindex = $bindex;
        $rt.throw($proc, $value)?;
        reset_context!($proc, $context, $index, $bindex);
        continue;
    };
}

macro_rules! throw_error_message {
    ($rt: expr,$proc: expr,$msg: expr,$context: ident,$index: ident,$bindex: ident) => {
        let value = Process::allocate_string($proc, &$rt.state, $msg);
        throw!($rt, $proc, value, $context, $index, $bindex)
    };
}

macro_rules! enter_context {
    ($process: expr,$context: ident,$index: ident,$bindex: ident) => {
        $context.bindex = $bindex;
        $context.index = $index;
        reset_context!($process, $context, $index, $bindex);
    };
}

macro_rules! safepoint_and_reduce {
    ($rt: expr,$process: expr,$reductions: expr) => {
        /*if $rt.gc_safepoint($process) {
            return Ok(Value::from(VTag::Null));
        }*/
        match $rt.gc_safepoint($process) {
            Ok(()) => (),
            Err(_) => {
                return Ok(Value::from(VTag::Null));
            }
        }

        if $reductions > 0 {
            $reductions -= 1;
        } else {
            $rt.state.scheduler.schedule($process.clone());
            return Ok(Value::from(VTag::Null));
        }
    };
}

impl Runtime {
    pub fn run(&self, worker: &mut ProcessWorker, process: &Arc<Process>) -> Result<Value, Value> {
        let mut reductions = 1000;
        let mut index;
        let mut bindex;
        let mut context: Ptr<Context>;
        //assert!(process.context_ptr().raw.is_null() == false);
        reset_context!(process, context, index, bindex);
        macro_rules! catch {
            ($value: expr) => {
                match $value {
                    Ok(result) => result,
                    Err(e) => throw!(self, process, e, context, index, bindex),
                }
            };
            (str $value: expr) => {
                match $value {
                    Ok(result) => result,
                    Err(e) => throw_error_message!(self, process, &e, context, index, bindex),
                }
            };
        }
        let (return_value, top_context): (Result<Value, Value>, bool) = loop {
            let ins = {
                let block = &context.code[bindex];
                block.instructions[index]
            };
            index += 1;
            match ins {
                Instruction::LoadStack(dest, ss0) => {
                    let value = context
                        .stack
                        .get(ss0 as usize)
                        .map(|x| *x)
                        .unwrap_or(Value::from(VTag::Undefined));
                    context.set_register(dest, value);
                }
                Instruction::StoreStack(dest, ss0) => {
                    let value = context.get_register(dest);
                    context.stack[ss0 as usize] = value;
                }
                Instruction::Return(value) => {
                    let value = if let Some(value) = value {
                        context.get_register(value)
                    } else {
                        Value::from(VTag::Null)
                    };
                    self.clear_catch_tables(&context, process);
                    if !context.in_tail {
                        let top_level;
                        if context.terminate_upon_return && !context.in_tail {
                            top_level = context.parent.is_none();
                            break (Ok(value), top_level);
                        }
                        if let Some(dest) = context.return_register {
                            if let Some(parent) = context.parent {
                                parent.get().set_register(dest, value);
                            }
                        }

                        if process.pop_context() {
                            break (Ok(value), true);
                        }
                    }
                    reset_context!(process, context, index, bindex);
                    safepoint_and_reduce!(self, process, reductions);
                }
                Instruction::LoadNull(r) => context.set_register(r, Value::from(VTag::Null)),
                Instruction::LoadUndefined(r) => {
                    context.set_register(r, Value::from(VTag::Undefined))
                }
                Instruction::LoadInt(r, i) => context.set_register(r, Value::new_int(i as _)),
                Instruction::LoadNumber(r, f) => {
                    context.set_register(r, Value::new_double(f64::from_bits(f)))
                }
                Instruction::Move(to, from) => {
                    let v0 = context.get_register(from);
                    context.set_register(to, v0);
                }
                Instruction::LoadConst(r0, c) => {
                    let global: Value = context.module.get_global_at(c as _);
                    if global.is_null_or_undefined() {
                        panic!("Null or undefined global");
                    }
                    context.set_register(r0, global);
                }
                Instruction::LoadTrue(r) => context.set_register(r, Value::from(VTag::True)),
                Instruction::LoadFalse(r) => context.set_register(r, Value::from(VTag::False)),
                Instruction::LoadById(dest, object, id) => {
                    let global = context.module.get_global_at(id as _);
                    if global.is_null_or_undefined() {
                        panic!("Null or undefined Id");
                    }
                    let id = global.to_string();

                    let object = context.get_register(object);
                    let id = Arc::new(id);
                    let attr = object.lookup_attribute(&self.state, &id);
                    if let Some(value) = attr {
                        context.set_register(dest, value);
                    } else {
                        throw_error_message!(
                            self,
                            process,
                            &format!("Attribute '{}' not found", id),
                            context,
                            index,
                            bindex
                        );
                    }
                }
                Instruction::LoadByValue(dest, object, value) => {
                    let value = context.get_register(value);
                    let object = context.get_register(object);
                    if object.is_cell() {
                        if let CellValue::Array(ref mut array) = object.as_cell().get_mut().value {
                            if value.is_number() {
                                let x = value.to_number().floor() as usize;
                                if x >= array.len() {
                                    for _ in x..=array.len() {
                                        array.push(Value::from(VTag::Undefined));
                                    }
                                }

                                context.set_register(dest, value);
                                continue;
                            }
                        }
                    }
                    let id = value.to_string();
                    let id = Arc::new(id);
                    let attr = object.lookup_attribute(&self.state, &id);
                    if let Some(value) = attr {
                        context.set_register(dest, value);
                    } else {
                        throw_error_message!(
                            self,
                            process,
                            &format!("Attribute '{}' not found", id),
                            context,
                            index,
                            bindex
                        );
                    }
                }
                Instruction::LoadStaticById(r0, id) => {
                    let global = context.module.get_global_at(id as _);
                    if global.is_null_or_undefined() {
                        panic!("Null or undefined id");
                    }
                    let id = global.to_string();
                    let statics = self.state.static_variables.lock();
                    let value = statics.get(&id);
                    if let None = value {
                        throw_error_message!(
                            self,
                            process,
                            &format!("Static variable '{}' not found", id),
                            context,
                            index,
                            bindex
                        );
                    } else if let Some(var) = value {
                        context.set_register(r0, *var);
                    }
                    drop(statics);
                }
                Instruction::StoreById(object, value, id) => {
                    let global = context.module.get_global_at(id as _);
                    if global.is_null_or_undefined() {
                        panic!("Null or undefined Id");
                    }
                    let id = Arc::new(global.to_string());
                    let object = context.get_register(object);
                    let value = context.get_register(value);

                    object.add_attribute_barriered(&self.state, &process, id, value);
                }
                Instruction::StoreByValue(object, key, value) => {
                    let object = context.get_register(object);
                    let key = context.get_register(key);
                    let value = context.get_register(value);
                    if object.is_cell() {
                        if let CellValue::Array(ref mut array) = object.as_cell().get_mut().value {
                            if key.is_number() {
                                let idx = key.to_number().floor() as usize;
                                if idx >= array.len() {
                                    for _ in idx..=array.len() {
                                        array.push(Value::from(VTag::Undefined))
                                    }
                                }
                                process
                                    .local_data_mut()
                                    .heap
                                    .field_write_barrier(object.as_cell(), value);
                                process
                                    .local_data_mut()
                                    .heap
                                    .write_barrier(object.as_cell());
                                array[idx] = value;
                                continue;
                            }
                        }
                    }
                    let id = Arc::new(key.to_string());
                    object.add_attribute_barriered(&self.state, &process, id, value);
                }
                Instruction::Push(r) => {
                    let value = context.get_register(r);
                    context.stack.push(value);
                }
                Instruction::Pop(r) => {
                    let value = context.stack.pop().unwrap_or(Value::from(VTag::Undefined));
                    context.set_register(r, value);
                }
                Instruction::Branch(block) => {
                    bindex = block as usize;
                    index = 0
                }
                Instruction::ConditionalBranch(r, if_true, if_false) => {
                    let value = context.get_register(r);
                    if value.to_boolean() {
                        bindex = if_true as _;
                    } else {
                        bindex = if_false as _;
                    }
                    index = 0;
                }
                Instruction::MakeEnv(function, count) => {
                    let mut upvalues = vec![];
                    for _ in 0..count {
                        upvalues.push(context.stack.pop().unwrap());
                    }

                    let function = context.get_register(function);
                    if function.is_cell() {
                        match function.as_cell().get_mut().value {
                            CellValue::Function(ref mut f) => {
                                let fun = function.as_cell();
                                for upvalue in upvalues.iter() {
                                    process
                                        .local_data_mut()
                                        .heap
                                        .field_write_barrier(fun, *upvalue);
                                }
                                f.upvalues = upvalues;
                            }
                            _ => {
                                panic!(
                                    "MakeEnv: Function expected, found '{}'",
                                    function.to_string()
                                );
                            }
                        }
                    } else {
                        panic!(
                            "MakeEnv: Function expected, found '{}'",
                            function.to_string()
                        );
                    }
                }
                Instruction::Throw(r) => {
                    let value = context.get_register(r);
                    throw!(self, process, value, context, index, bindex);
                }
                Instruction::CatchBlock(register, block) => {
                    let entry = CatchTable {
                        context: context,
                        jump_to: block,
                        register: register,
                    };

                    process.local_data_mut().catch_tables.push(entry);
                }
                Instruction::Call(dest, function, argc)
                | Instruction::TailCall(dest, function, argc) => {
                    let function = context.get_register(function);
                    if !function.is_cell() {
                        throw_error_message!(
                            self,
                            process,
                            &format!(
                                "Cannot invoke '{}' value. (op {:?})",
                                function.to_string(),
                                ins
                            ),
                            context,
                            index,
                            bindex
                        );
                    }
                    let cell = function.as_cell();
                    let maybe_function = cell.function_value();
                    let function: &Function = match maybe_function {
                        Ok(function) => function,
                        Err(_) => {
                            throw_error_message!(
                                self,
                                process,
                                &format!("Cannot invoke '{}' value.", function.to_string()),
                                context,
                                index,
                                bindex
                            );
                        }
                    };
                    /*if argc as i32 != function.argc && function.argc != -1 {
                        throw_error_message!(
                            self,
                            process,
                            &format!(
                                "Expected '{}' argument(s) to function {}, found '{}'",
                                function.argc, function.name, argc
                            ),
                            context,
                            index,
                            bindex
                        );
                    }*/
                    let mut args = vec![];

                    for _ in 0..argc {
                        args.push(context.get().stack.pop().unwrap());
                    }

                    if let Some(native_fn) = function.native {
                        let result =
                            native_fn(&self.state, process, Value::from(VTag::Undefined), &args);
                        if let Err(err) = result {
                            throw!(self, process, err, context, index, bindex);
                        }
                        let result = result.unwrap();
                        match result {
                            ReturnValue::Value(value) => context.set_register(dest, value),
                            ReturnValue::SuspendProcess => {
                                context.index = index - 1;
                                context.bindex = bindex;
                                for arg in args.iter().rev() {
                                    context.stack.push(*arg);
                                }
                                return Ok(Value::from(VTag::Null));
                            }
                            ReturnValue::YieldProcess => {
                                self.state.scheduler.schedule(process.clone());
                                return Ok(Value::from(VTag::Null));
                            }
                        }
                    } else {
                        if let Instruction::Call { .. } = ins {
                            let mut new_context = Context::new();
                            new_context.return_register = Some(dest);
                            new_context.stack = args;
                            new_context.function = cell;
                            new_context.module = function.module.clone();
                            new_context.n = context.n + 1;
                            new_context.code = function.code.clone();
                            new_context.terminate_upon_return = false;
                            process.push_context(new_context);
                            enter_context!(process, context, index, bindex);
                        } else {
                            /*assert!(
                                !process.pop_context(),
                                "Tail calls cannot be done in global context"
                            );*/
                            //process.push_context_ptr(context);
                            /*let mut prev_ctx = Context::new();
                            prev_ctx.bindex = bindex;
                            prev_ctx.index = index;
                            prev_ctx.registers = context.registers;
                            prev_ctx.return_register = context.return_register;
                            prev_ctx.this = context.this;
                            prev_ctx.code = context.code.clone();
                            prev_ctx*/
                            let mut new_context = context;
                            new_context.return_register = Some(dest);
                            new_context.stack = args;
                            new_context.function = cell;
                            new_context.module = function.module.clone();
                            //new_context.n = context.n + 1;
                            new_context.code = function.code.clone();
                            new_context.index = 0;
                            new_context.in_tail = true;
                            new_context.bindex = 0;
                            new_context.terminate_upon_return = false;
                            //process.push_context(new_context);
                            reset_context!(process, context, index, bindex);
                            //enter_context!(process, context, index, bindex);
                        }
                    }
                }
                Instruction::VirtCall(dest, function, this, argc) => {
                    let function = context.get_register(function);
                    if !function.is_cell() {
                        throw_error_message!(
                            self,
                            process,
                            &format!("Cannot invoke '{}' value.", function.to_string()),
                            context,
                            index,
                            bindex
                        );
                    }
                    let cell = function.as_cell();
                    let maybe_function = cell.function_value();
                    let function: &Function = match maybe_function {
                        Ok(function) => function,
                        Err(_) => {
                            throw_error_message!(
                                self,
                                process,
                                &format!("Cannot invoke '{}' value.", function.to_string()),
                                context,
                                index,
                                bindex
                            );
                        }
                    };
                    if argc as i32 != function.argc && function.argc != -1 {
                        throw_error_message!(
                            self,
                            process,
                            &format!(
                                "Expected '{}' argument(s) to function {}, found '{}'",
                                function.argc, function.name, argc
                            ),
                            context,
                            index,
                            bindex
                        );
                    }
                    let mut args = vec![];

                    for _ in 0..argc {
                        args.push(context.stack.pop().unwrap());
                    }
                    let this = context.get_register(this);

                    if let Some(native_fn) = function.native {
                        let result = native_fn(&self.state, process, this, &args);
                        if let Err(err) = result {
                            throw!(self, process, err, context, index, bindex);
                        }
                        let result = result.unwrap();
                        match result {
                            ReturnValue::Value(value) => context.set_register(dest, value),
                            ReturnValue::SuspendProcess => {
                                context.index = index - 1;
                                context.bindex = bindex;
                                for arg in args.iter().rev() {
                                    context.stack.push(*arg);
                                }
                                return Ok(Value::from(VTag::Null));
                            }
                            ReturnValue::YieldProcess => {
                                self.state.scheduler.schedule(process.clone());
                                return Ok(Value::from(VTag::Null));
                            }
                        }
                    } else {
                        let mut new_context = Context::new();
                        new_context.return_register = Some(dest);
                        new_context.stack = args;
                        new_context.function = cell;
                        new_context.module = function.module.clone();
                        new_context.n = context.n + 1;
                        new_context.this = this;
                        new_context.code = function.code.clone();
                        new_context.terminate_upon_return = false;
                        process.push_context(new_context);
                        enter_context!(process, context, index, bindex);
                    }
                }
                Instruction::LoadUpvalue(r, slot) => {
                    let value = context
                        .function
                        .function_value()
                        .unwrap()
                        .upvalues
                        .get(slot as usize)
                        .map(|x| *x)
                        .unwrap();
                    context.set_register(r, value);
                }
                Instruction::Gc => match process.local_data_mut().heap.collect_garbage(process) {
                    Ok(_) => (),
                    Err(_) => return Ok(Value::from(VTag::Null)),
                },
                Instruction::GcSafepoint => match self.gc_safepoint(process) {
                    Ok(_) => (),
                    Err(_) => return Ok(Value::from(VTag::Null)),
                },
                Instruction::New(dest, function, argc) => {
                    let function = context.get_register(function);
                    if !function.is_cell() {
                        throw_error_message!(
                            self,
                            process,
                            &format!("Cannot invoke '{}' value.", function.to_string()),
                            context,
                            index,
                            bindex
                        );
                    }
                    let cell = function.as_cell();
                    let maybe_function = cell.function_value();
                    let (function, prototype): (Arc<Function>, CellPointer) = match maybe_function {
                        Ok(function) => (
                            function.clone(),
                            cell.lookup_attribute_in_self(
                                &self.state,
                                &Arc::new("prototype".to_owned()),
                            )
                            .unwrap()
                            .as_cell(),
                        ),
                        Err(_) => {
                            let ctor = Arc::new("constructor".to_owned());
                            if let Some(ctor) =
                                function.lookup_attribute_in_self(&self.state, &ctor)
                            {
                                if ctor.is_cell() {
                                    if let Ok(ctor) = ctor.as_cell().function_value() {
                                        (ctor.clone(), cell)
                                    } else {
                                        throw_error_message!(
                                            self,
                                            process,
                                            &format!(
                                                "Cannot invoke constructor on '{}' value.",
                                                function.to_string()
                                            ),
                                            context,
                                            index,
                                            bindex
                                        );
                                    }
                                } else {
                                    throw_error_message!(
                                        self,
                                        process,
                                        &format!(
                                            "Cannot invoke constructor on '{}' value.",
                                            function.to_string()
                                        ),
                                        context,
                                        index,
                                        bindex
                                    );
                                }
                            } else {
                                throw_error_message!(
                                    self,
                                    process,
                                    &format!(
                                        "Cannot invoke constructor on '{}' value.",
                                        function.to_string()
                                    ),
                                    context,
                                    index,
                                    bindex
                                );
                            }
                        }
                    };
                    if argc as i32 != function.argc && function.argc != -1 {
                        throw_error_message!(
                            self,
                            process,
                            &format!(
                                "Expected '{}' argument(s) to function {}, found '{}'",
                                function.argc, function.name, argc
                            ),
                            context,
                            index,
                            bindex
                        );
                    }
                    let mut args = vec![];

                    for _ in 0..argc {
                        args.push(context.stack.pop().unwrap());
                    }
                    let this = Process::allocate(
                        process,
                        Cell::with_prototype(CellValue::None, prototype),
                    );

                    if let Some(native_fn) = function.native {
                        let result = native_fn(&self.state, process, this, &args);
                        if let Err(err) = result {
                            throw!(self, process, err, context, index, bindex);
                        }
                        let result = result.unwrap();
                        match result {
                            ReturnValue::Value(value) => context.set_register(dest, value),
                            ReturnValue::SuspendProcess => {
                                context.index = index - 1;
                                context.bindex = bindex;
                                for arg in args.iter().rev() {
                                    context.stack.push(*arg);
                                }
                                return Ok(Value::from(VTag::Null));
                            }
                            ReturnValue::YieldProcess => {
                                self.state.scheduler.schedule(process.clone());
                                return Ok(Value::from(VTag::Null));
                            }
                        }
                    } else {
                        let mut new_context = Context::new();
                        new_context.return_register = Some(dest);
                        new_context.stack = args;
                        new_context.function = cell;
                        new_context.module = function.module.clone();
                        new_context.n = context.n + 1;
                        new_context.this = this;
                        new_context.code = function.code.clone();
                        new_context.terminate_upon_return = false;
                        process.push_context(new_context);
                        enter_context!(process, context, index, bindex);
                    }
                }
                Instruction::Binary(op, dest, lhs, rhs) => {
                    let lhs = context.get_register(lhs);
                    let rhs = context.get_register(rhs);
                    assert!(!lhs.is_empty() && !rhs.is_empty());
                    if lhs.is_number() && rhs.is_number() {
                        let lhs = lhs.to_number();
                        let rhs = rhs.to_number();
                        let result = match op {
                            BinOp::Add => Value::new_double(lhs + rhs),
                            BinOp::Sub => Value::new_double(lhs - rhs),
                            BinOp::Div => Value::new_double(lhs / rhs),
                            BinOp::Mul => Value::new_double(lhs * rhs),
                            BinOp::Mod => Value::new_double(lhs % rhs),
                            BinOp::Lsh => {
                                Value::new_double(((lhs.ceil() as i64) >> rhs.ceil() as i64) as f64)
                            }
                            BinOp::Rsh => {
                                Value::new_double(((lhs.ceil() as i64) << rhs.ceil() as i64) as f64)
                            }
                            BinOp::Equal => Value::from(lhs == rhs),
                            BinOp::NotEqual => Value::from(lhs != rhs),
                            BinOp::Greater => Value::from(lhs > rhs),
                            BinOp::Less => Value::from(lhs < rhs),
                            BinOp::GreaterOrEqual => Value::from(lhs >= rhs),
                            BinOp::LessOrEqual => Value::from(lhs <= rhs),
                            BinOp::And => {
                                Value::new_double(((lhs.ceil() as i64) & rhs.ceil() as i64) as f64)
                            }
                            BinOp::Or => {
                                Value::new_double(((lhs.ceil() as i64) | rhs.ceil() as i64) as f64)
                            }
                            BinOp::Xor => {
                                Value::new_double(((lhs.ceil() as i64) ^ rhs.ceil() as i64) as f64)
                            }
                        };
                        context.set_register(dest, result);
                    } else if lhs.is_bool() && rhs.is_bool() {
                        let lhs = lhs.to_boolean();
                        let rhs = rhs.to_boolean();
                        let result = match op {
                            BinOp::Add => Value::new_int(lhs as i32 + rhs as i32),
                            BinOp::Sub => Value::new_int(lhs as i32 + rhs as i32),
                            BinOp::Div => Value::new_int(lhs as i32 + rhs as i32),
                            BinOp::Mul => Value::new_int(lhs as i32 * rhs as i32),
                            BinOp::Equal => Value::from(lhs == rhs),
                            BinOp::NotEqual => Value::from(lhs != rhs),
                            BinOp::Greater => Value::from(lhs > rhs),
                            BinOp::Less => Value::from(lhs < rhs),
                            BinOp::LessOrEqual => Value::from(lhs <= rhs),
                            BinOp::GreaterOrEqual => Value::from(lhs >= rhs),
                            _ => Value::from(VTag::Null),
                        };
                        context.set_register(dest, result);
                    } else if lhs.is_null_or_undefined() && rhs.is_null_or_undefined() {
                        context.set_register(dest, Value::from(lhs == rhs));
                    } else if lhs.is_cell() && rhs.is_cell() {
                        let lhs = lhs.as_cell();
                        let rhs = rhs.as_cell();
                        if lhs.is_string() && rhs.is_string() {
                            match op {
                                BinOp::Add => {
                                    context.set_register(
                                        dest,
                                        Process::allocate_string(
                                            process,
                                            &self.state,
                                            &format!("{}{}", lhs, rhs),
                                        ),
                                    );
                                }
                                BinOp::Equal => context.set_register(
                                    dest,
                                    Value::from(lhs.to_string() == rhs.to_string()),
                                ),
                                BinOp::NotEqual => context.set_register(
                                    dest,
                                    Value::from(lhs.to_string() != rhs.to_string()),
                                ),
                                BinOp::Greater => context.set_register(
                                    dest,
                                    Value::from(lhs.to_string().len() > rhs.to_string().len()),
                                ),
                                BinOp::Less => context.set_register(
                                    dest,
                                    Value::from(lhs.to_string().len() < rhs.to_string().len()),
                                ),
                                _ => unimplemented!(),
                            }
                            continue;
                        }
                        if lhs.is_string() {
                            match op {
                                BinOp::Add => {
                                    context.set_register(
                                        dest,
                                        Process::allocate_string(
                                            process,
                                            &self.state,
                                            &format!("{}{}", lhs, rhs),
                                        ),
                                    );
                                }
                                _ => context.set_register(dest, Value::from(false)),
                            }
                            continue;
                        } else if lhs.is_array() {
                            match op {
                                BinOp::Add => match &rhs.get().value {
                                    CellValue::Array(array) => {
                                        let mut new_array =
                                            lhs.array_value().map(|x| x.clone()).unwrap();
                                        new_array.extend(array.iter().copied());
                                        context.set_register(
                                            dest,
                                            Process::allocate(
                                                process,
                                                Cell::with_prototype(
                                                    CellValue::Array(new_array),
                                                    self.state.array_prototype.as_cell(),
                                                ),
                                            ),
                                        );
                                        continue;
                                    }
                                    _ => {
                                        context.set_register(
                                            dest,
                                            Process::allocate_string(
                                                process,
                                                &self.state,
                                                &format!("{}{}", lhs, rhs),
                                            ),
                                        );
                                        continue;
                                    }
                                },
                                BinOp::Equal => context.set_register(dest, Value::from(lhs == rhs)),
                                BinOp::NotEqual => {
                                    context.set_register(dest, Value::from(lhs != rhs))
                                }
                                _ => context.set_register(dest, Value::new_double(0.0)),
                            }
                        }
                        let result = match op {
                            BinOp::Equal => Value::from(lhs == rhs),
                            BinOp::NotEqual => Value::from(lhs != rhs),
                            BinOp::Add => Process::allocate_string(
                                process,
                                &self.state,
                                &format!("{}{}", lhs, rhs),
                            ),
                            _ => Value::new_double(std::f64::NAN),
                        };
                        context.set_register(dest, result)
                    } else if lhs.is_cell() {
                        let result = match op {
                            BinOp::Equal => Value::from(lhs == rhs),
                            BinOp::NotEqual => Value::from(lhs != rhs),
                            BinOp::Add => Process::allocate_string(
                                process,
                                &self.state,
                                &format!("{}{}", lhs, rhs),
                            ),
                            _ => Value::new_double(std::f64::NAN), // TODO: Do we really need to use NaN if operation is not supported on current type?
                        };
                        context.set_register(dest, result);
                    } else {
                        let result = match op {
                            BinOp::Equal => Value::from(lhs == rhs),
                            BinOp::NotEqual => Value::from(lhs != rhs),
                            BinOp::Add => Process::allocate_string(
                                process,
                                &self.state,
                                &format!("{}{}", lhs, rhs),
                            ),
                            _ => Value::new_double(std::f64::NAN),
                        };
                        context.set_register(dest, result);
                    }
                }
                Instruction::Unary(op, dest, lhs) => {
                    let lhs = context.get_register(lhs);
                    let result = match op {
                        UnaryOp::Neg => {
                            if lhs.is_number() {
                                Value::new_double(-lhs.to_number())
                            } else if lhs.is_bool() {
                                Value::new_double(-(lhs.to_boolean() as i32 as f64))
                            } else {
                                Value::new_double(std::f64::NAN)
                            }
                        }
                        UnaryOp::Not => {
                            if lhs.is_bool() {
                                Value::from(!lhs.to_boolean())
                            } else if lhs.is_number() {
                                Value::from(!(lhs.to_number() == 0.0 || lhs.to_number().is_nan()))
                            } else if lhs.is_cell() {
                                Value::from(false)
                            } else if lhs.is_null_or_undefined() {
                                Value::from(!false)
                            } else {
                                Value::from(false)
                            }
                        }
                    };
                    context.set_register(dest, result);
                }
                Instruction::LoadCurrentModule(to) => {
                    let module = Value::from(Process::allocate(
                        process,
                        Cell::with_prototype(
                            CellValue::Module(context.module.clone()),
                            self.state.module_prototype.as_cell(),
                        ),
                    ));
                    context.set_register(to, module);
                }
                Instruction::LoadThis(to) => {
                    let this = context.this;
                    context.set_register(to, this);
                }
                Instruction::SetThis(x) => {
                    let new_this = context.get_register(x);
                    context.this = new_this;
                }

                x => panic!("{:?}", x),
            }
        };
        let ret = return_value?;
        if top_context {
            if process.is_pinned() {
                worker.leave_exclusive_mode();
            }
            process.terminate(&self.state);

            if process.is_main() {
                self.terminate();
            }
        }

        Ok(ret)
    }
    /// ReturnValues true if a process is garbage collected.
    pub fn gc_safepoint(&self, process: &Arc<Process>) -> Result<(), bool> {
        if !process.local_data().heap.should_collect() {
            return Ok(());
        }
        process.local_data_mut().heap.collect_garbage(process)
    }

    pub fn schedule_main_process(&self, proc: Arc<Process>) {
        proc.set_main();
        self.state.scheduler.schedule_on_main_thread(proc);
    }
    pub fn schedule(&self, proc: Arc<Process>) {
        self.state.scheduler.schedule(proc);
    }
    pub fn throw(&self, process: &Arc<Process>, value: Value) -> Result<Value, Value> {
        if let Some(table) = process.local_data_mut().catch_tables.pop() {
            let mut catch_ctx = table.context.replace(Context::new());
            catch_ctx.set_register(table.register, value);
            catch_ctx.bindex = table.jump_to as _;
            process.push_context(catch_ctx);

            Ok(Value::empty())
        } else {
            return Err(value);
        }
    }

    pub fn run_with_error_handling(&self, worker: &mut ProcessWorker, process: &Arc<Process>) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if let Err(error) = self.run(worker, process) {
                self.run_default_panic(process, &error);
            }
        }));

        if let Err(error) = result {
            let _ = if error.is::<Value>() {
                self.run_default_panic(process, &*error.downcast::<Value>().unwrap());
            } else if error.is::<String>() {
                self.run_default_panic(
                    process,
                    &Value::from(Process::allocate_string(
                        process,
                        &self.state,
                        &error.downcast::<String>().unwrap(),
                    )),
                );
            } else {
                self.run_default_panic(
                    process,
                    &Value::from(Process::allocate_string(
                        process,
                        &self.state,
                        "Unknown error",
                    )),
                );
            };
        }
    }
    pub fn clear_catch_tables(&self, exiting: &Ptr<Context>, proc: &Arc<Process>) {
        proc.local_data_mut()
            .catch_tables
            .retain(|ctx| ctx.context.n < exiting.n);
    }
    pub fn run_default_panic(&self, proc: &Arc<Process>, message: &Value) {
        runtime_panic(proc, message);
        self.terminate();
    }

    pub fn terminate(&self) {
        self.state.scheduler.terminate();
        self.state.timeout_worker.terminate();
    }
}

pub fn runtime_panic(process: &Arc<Process>, message: &Value) {
    let mut frames = vec![];
    let mut buffer = String::new();
    for ctx in process.local_data().context.contexts() {
        frames.push(format!(
            "in module \"{}\": {}",
            ctx.module.name,
            ctx.function.function_value().unwrap().name
        ));
    }

    frames.reverse();
    buffer.push_str("Stack trace (the most recent call comes last):");

    for (index, line) in frames.iter().enumerate() {
        buffer.push_str(&format!("\n  {}: {}", index, line));
    }

    buffer.push_str(&format!(
        "\nPanic '{}' in process {:#x}",
        message.to_string(),
        process.as_ptr() as usize,
    ));

    eprintln!("{}", buffer);
}
