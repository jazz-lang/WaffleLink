use super::cell::*;
use super::process::*;
use super::scheduler::process_worker::ProcessWorker;
use super::state::*;
use super::value::*;
use crate::util::arc::Arc;

pub extern "C" fn type_error(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    _: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    let msg = if let Some(arg) = arguments.get(0) {
        format!("TypeError: {}", arg.to_string())
    } else {
        format!("TypeError")
    };
    let proto = state
        .static_variables
        .lock()
        .get("TypeError")
        .copied()
        .unwrap()
        .as_cell();
    let mut cell = Cell::with_prototype(CellValue::None, proto);
    cell.add_attribute(
        Arc::new("message".to_owned()),
        Process::allocate_string(process, state, &msg),
    );
    Ok(ReturnValue::Value(Value::from(Process::allocate(
        process, cell,
    ))))
}

pub extern "C" fn exception_to_string(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    this: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    let message = this.lookup_attribute(state, &Arc::new("message".to_owned()));
    let message = if let None = message {
        "Unknown exception".to_owned()
    } else if let Some(message) = message {
        message.to_string()
    } else {
        unreachable!()
    };

    Ok(ReturnValue::Value(Value::from(Process::allocate_string(
        process, state, &message,
    ))))
}

pub fn initialize_exception(state: &RcState) {
    let mut vars = state.static_variables.lock();
    let exception = state.allocate(Cell::with_prototype(
        CellValue::None,
        state.object_prototype.as_cell(),
    ));
    exception.add_attribute_without_barrier(
        state,
        Arc::new("toString".to_owned()),
        Value::from(state.allocate_native_fn(exception_to_string, "toString", 0)),
    );
    vars.insert("Exception".to_owned(), Value::from(exception));
    let cell = state.allocate(Cell::with_prototype(CellValue::None, exception.as_cell()));
    cell.add_attribute_without_barrier(
        state,
        Arc::new("constructor".to_owned()),
        Value::from(state.allocate_native_fn(type_error, "constructor", -1)),
    );

    vars.insert("TypeError".to_owned(), Value::from(cell));
}
