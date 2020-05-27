use super::fiber::*;
use super::gc::*;
use super::value::*;
use super::*;
pub fn interpret(mut f: Handle<Fiber>) -> Result<Value, Value> {
    Ok(Value::undefined())
}
