use super::cell::*;
use super::process::*;
use super::state::*;
use super::*;
use super::value::*;
use crate::util::arc::Arc;

pub extern "C" fn require(
    state: &RcState,
    process: &Arc<Process>,
    _: Value,
    arguments: &[Value]
) -> Result<ReturnValue,Value> {
    let rt: &Runtime = &RUNTIME;
    let mut registry = rt.registry.lock();
    let (module,loaded) = registry.load("",&arguments[0].to_string()).map_err(|err| Value::from(Process::allocate_string(process,state,&err)))?;
    unimplemented!()
}
