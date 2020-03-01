use super::cell::*;
use super::process::*;
use super::scheduler::process_worker::ProcessWorker;
use super::state::*;
use super::value::*;
use crate::util::arc::Arc;

pub extern "C" fn constructor(
    _: &mut ProcessWorker,
    state: &RcState,
    process: &Arc<Process>,
    this: Value,
    _: &[Value],
) -> Result<ReturnValue, Value> {
    if !this.is_cell() {
        let object = Process::allocate(
            process,
            Cell::with_prototype(CellValue::None, state.object_prototype.as_cell()),
        );

        return Ok(ReturnValue::Value(Value::from(object)));
    } else {
        let cell = this.as_cell();
        cell.get_mut()
            .set_prototype(state.object_prototype.as_cell());
        cell.get_mut().value = CellValue::None;
        return Ok(ReturnValue::Value(this));
    }
}

pub fn initialize_object(state: &RcState) {
    let mut lock = state.static_variables.lock();
    let object = state.object_prototype.as_cell();
    object.add_attribute_without_barrier(
        &Arc::new("constructor".to_owned()),
        Value::from(state.allocate_native_fn(constructor, "constructor", -1)),
    );

    lock.insert("Object".to_owned(), Value::from(object));
}
