use super::cell::*;
use super::process::*;
use super::scheduler::process_worker::ProcessWorker;
use super::state::*;
use super::value::*;
use super::*;
use crate::interpreter::context::*;
use crate::util::arc::Arc;
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
                terminate_upon_return: true,
                return_register: None,
                n: process.context_ptr().n,
                in_tail: false,
                index: 0,
                bindex: 0,
                parent: None,
                function: main_fn.as_cell(),
                code: function.code.clone(),
                module: function.module.clone(),
                registers: [Value::from(VTag::Undefined); 32],
                stack: vec![],
                this: Value::from(VTag::Undefined),
            };
            module.as_cell().module_value_mut().unwrap().exports = Process::allocate(
                process,
                Cell::with_prototype(CellValue::None, state.object_prototype.as_cell()),
            );
            process.push_context(ctx);
            let _ = RUNTIME.run(worker, process)?;
            return Ok(ReturnValue::Value(
                module.as_cell().module_value().unwrap().exports,
            ));
        }
        _ => panic!("Function expected"),
    };
}

pub extern "C" fn __start(
    worker: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    _: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    let home_dir = format!(
        "{}/.waffle/builtins/",
        dirs::home_dir().unwrap().to_str().unwrap().to_owned()
    );
    require(
        worker,
        state,
        process,
        Value::empty(),
        &[Value::from(
            state.intern(&format!("{}/Array.wfl", home_dir)),
        )],
    )?;
    require(
        worker,
        state,
        process,
        Value::empty(),
        &[Value::from(state.intern(&format!("{}/Math.wfl", home_dir)))],
    )?;
    require(
        worker,
        state,
        process,
        Value::empty(),
        &[Value::from(state.intern(&format!("{}/Core.wfl", home_dir)))],
    )?;

    Ok(ReturnValue::Value(Value::from(VTag::Null)))
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

pub fn initialize_core(state: &RcState) {
    let mut lock = state.static_variables.lock();
    let require = state.allocate_native_fn(require, "require", 1);
    let start = state.allocate_native_fn(__start, "__start__", 0);
    let instanceof = state.allocate_native_fn(instanceof, "instanceof", 2);
    lock.insert("require".to_owned(), Value::from(require));
    lock.insert("__start__".to_owned(), Value::from(start));
    lock.insert("instanceof".to_owned(), Value::from(instanceof));
}
