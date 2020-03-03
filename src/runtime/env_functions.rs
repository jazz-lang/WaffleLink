use super::cell::*;
use super::exception::*;
use super::process::*;
use super::scheduler::process_worker::*;
use super::state::*;
use super::value::*;
use crate::util::arc::Arc;

pub extern "C" fn get_home(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    _: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    Ok(ReturnValue::Value(Value::from(state.intern_string(
        dirs::home_dir().unwrap().to_str().unwrap().to_owned(),
    ))))
}

pub fn initialize_env(state: &RcState) {
    let mut lock = state.static_variables.lock();
    let mut env = Cell::with_prototype(CellValue::None, state.object_prototype.as_cell());
    env.add_attribute(
        Arc::new("getHome".to_owned()),
        Value::from(state.allocate_native_fn(get_home, "getHome", 0)),
    );
    lock.insert("env".to_owned(), Value::from(state.allocate(env)));
}
