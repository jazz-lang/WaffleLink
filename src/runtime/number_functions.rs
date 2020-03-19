use super::cell::*;
use super::process::*;
use super::scheduler::process_worker::*;
use super::state::*;
use super::value::*;
use crate::util::arc::Arc;

native_fn!(
    _worker,state,proc => constructor (...args) {
        match args.len() {
            1 => {
                let string = args[0].to_string();
                if let Ok(value) = string.parse::<i32>() {
                    Ok(ReturnValue::Value(Value::new_int(value)))
                } else if let Ok(value) = string.parse::<f64>() {
                    Ok(ReturnValue::Value(Value::new_double(value)))
                } else {
                    Ok(ReturnValue::Value(Value::from(VTag::Null))) // TODO: Maybe it will be better to throw exception there?
                }
            }
            _ => Ok(ReturnValue::Value(Value::new_int(0))),
            
        }
    }
);

pub fn initialize_number(state: &RcState) {
    let mut lock = state.static_variables.lock();

    let number = state.number_prototype.as_cell();
    number.add_attribute_without_barrier(
        &Arc::new("constructor".to_owned()),
        Value::from(state.allocate_native_fn(constructor, "constructor", -1)),
    );
    lock.insert("Number".to_owned(), Value::from(number));
}
