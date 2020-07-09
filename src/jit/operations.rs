use crate::object::*;
use crate::value::*;
pub extern "C" fn operation_value_add(_global_object: Ref<Obj>, _op1: Value, _op2: Value) -> Value {
    Value::undefined()
}
