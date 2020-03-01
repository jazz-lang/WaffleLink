use super::cell::*;
use super::process::*;
use super::scheduler::process_worker::ProcessWorker;
use super::state::*;
use super::value::*;
use super::*;
use crate::interpreter::context::*;
use crate::util::arc::Arc;

pub extern "C" fn load(
    _worker: &mut ProcessWorker,
    _state: &RcState,
    _process: &Arc<Process>,
    _: Value,
    _arguments: &[Value],
) -> Result<ReturnValue, Value> {
    unimplemented!()
}

pub extern "C" fn exports(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    this: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    let module = if this == state.module_prototype {
        process.context_ptr().module.clone()
    } else {
        this.as_cell().module_value().unwrap().clone()
    };

    Ok(ReturnValue::Value(module.exports))
}

pub fn initialize_module(state: &RcState) {
    let mut lock = state.static_variables.lock();
    state.module_prototype.add_attribute_without_barrier(
        state,
        Arc::new("exports".to_owned()),
        Value::from(state.allocate_native_fn(exports, "exports", 0)),
    );
    lock.insert("Module".to_owned(), state.module_prototype);
}
