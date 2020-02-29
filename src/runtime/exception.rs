use super::cell::*;
use super::process::*;
use super::scheduler::process_worker::ProcessWorker;
use super::state::*;
use super::value::*;
use crate::util::arc::Arc;

pub extern "C" fn type_error(
    _: &mut ProcessWorker,
    state: &RcState,
    proc: &Arc<Process>,
    _: Value,
    arguments: &[Value],
) -> Result<ReturnValue, Value> {
    let msg = if let Some(arg) = arguments.get(0) {
        format!("TypeError: {}",arg.to_string())
    } else {
        format!("TypeError")
    };

    //let cell = Cell::with_prototype(CellValue::None)
    unimplemented!()

}

pub fn initialize_exception(state: &RcState) {}
