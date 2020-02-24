use super::cell::*;
use super::process::*;
use super::state::*;
use super::value::*;
use crate::util::arc::Arc;

pub extern "C" fn writeln(
    _: &RcState,
    _: &Arc<Process>,
    _: Value,
    arguments: &[Value],
) -> Result<Return, Value> {
    for value in arguments.iter() {
        print!("{}", value);
    }
    println!();
    Ok(Return::Value(Value::from(VTag::Null)))
}

pub fn initialize_io(state: &RcState) {
    let io = Cell::with_prototype(CellValue::None, state.object_prototype.as_cell());
    let io = state.allocate(io);
    state.static_variables.lock().insert("io".to_owned(), io);
    let name = Arc::new("writeln".to_owned());
    let writeln = state.allocate_native_fn_with_name(writeln, "writeln", -1);
    io.as_cell()
        .add_attribute_without_barrier(&name, Value::from(writeln));
}
