pub mod context;

use crate::bytecode::instruction::*;
use crate::heap::*;
use crate::runtime::*;
use crate::util::arc::Arc;
use crate::util::ptr::Ptr;
use cell::*;
use context::*;
use gc_pool::Collection;
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
        let value = $proc.allocate_string(&$rt.state, $msg);
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
        if $rt.gc_safepoint($process) {
            return Ok(Value::from(VTag::Null));
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
        let return_value: Result<Value, Value> = loop {
            let block = unsafe { context.code.get_unchecked(bindex) };
            let ins = unsafe { block.instructions.get_unchecked(index) };
            index += 1;
            match *ins {
                Instruction::Return(value) => {
                    let value = if let Some(value) = value {
                        context.get_register(value)
                    } else {
                        Value::from(VTag::Null)
                    };
                    self.clear_catch_tables(&context, process);
                    if context.terminate_upon_return {
                        break Ok(value);
                    }

                    if process.pop_context() {
                        break Ok(value);
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
                            CellValue::Function(ref mut f) => f.upvalues = upvalues,
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
                Instruction::Call(dest, function, argc) => {
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

                    if let Some(native_fn) = function.native {
                        let result =
                            native_fn(&self.state, process, Value::from(VTag::Undefined), &args);
                        if let Err(err) = result {
                            throw!(self, process, err, context, index, bindex);
                        }
                        let result = result.unwrap();
                        match result {
                            Return::Value(value) => context.set_register(dest, value),
                            Return::SuspendProcess => {
                                break Ok(Value::from(VTag::Null));
                            }
                            Return::YieldProcess => {
                                self.state.scheduler.schedule(process.clone());
                                break Ok(Value::from(VTag::Null));
                            }
                        }
                    } else {
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
                            Return::Value(value) => context.set_register(dest, value),
                            Return::SuspendProcess => {
                                break Ok(Value::from(VTag::Null));
                            }
                            Return::YieldProcess => {
                                self.state.scheduler.schedule(process.clone());
                                break Ok(Value::from(VTag::Null));
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
                Instruction::Gc => {
                    context.bindex = bindex;
                    context.index = index;
                    self.state
                        .gc_pool
                        .schedule(Collection::new(process.clone()));
                    return Ok(Value::empty());
                }
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
                    let this = process.allocate(Cell::with_prototype(CellValue::None, prototype));

                    if let Some(native_fn) = function.native {
                        let result = native_fn(&self.state, process, this, &args);
                        if let Err(err) = result {
                            throw!(self, process, err, context, index, bindex);
                        }
                        let result = result.unwrap();
                        match result {
                            Return::Value(value) => context.set_register(dest, value),
                            Return::SuspendProcess => {
                                return Ok(Value::from(VTag::Null));
                            }
                            Return::YieldProcess => {
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

                _ => unimplemented!(),
            }
        };

        let ret = return_value?;
        if process.is_pinned() {
            worker.leave_exclusive_mode();
        }

        process.terminate(&self.state);

        if process.is_main() {
            self.terminate();
        }

        Ok(ret)
    }
    /// Returns true if a process should be suspended for garbage collection.
    pub fn gc_safepoint(&self, process: &Arc<Process>) -> bool {
        if !process.local_data().heap.should_collect() {
            return false;
        }
        self.state
            .gc_pool
            .schedule(Collection::new(process.clone()));
        true
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
            if let Ok(message) = error.downcast::<String>() {
                self.run_default_panic(
                    process,
                    &Value::from(process.allocate_string(&self.state, &message)),
                );
            } else {
                self.run_default_panic(
                    process,
                    &Value::from(process.allocate_string(&self.state, "Unknown error")),
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
        self.state.gc_pool.terminate();
        self.state.timeout_worker.terminate();
    }
}

pub fn runtime_panic(process: &Arc<Process>, message: &Value) {
    let mut frames = vec![];
    let mut buffer = String::new();
    for ctx in process.local_data().context.contexts() {
        frames.push(format!(
            "\"{}\" in {}",
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
