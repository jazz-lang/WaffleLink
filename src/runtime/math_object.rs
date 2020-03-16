/*
*   Copyright (c) 2020 Adel Prokurov
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/

use super::cell::*;
use super::process::*;
use super::scheduler::process_worker::ProcessWorker;
use super::state::*;
use super::value::*;
use crate::util::arc::Arc;

native_fn!(
    _worker,_state,_proc => math_sin (num) Ok(ReturnValue::Value(Value::new_double(num.to_number().sin())))
);

native_fn!(
  _worker,_state,_proc => math_cos(num) Ok(ReturnValue::Value(Value::new_double(num.to_number().cos())))
);

native_fn!(
  _worker,_state,_proc => math_tan(num) Ok(ReturnValue::Value(Value::new_double(num.to_number().tan())))
);

native_fn!(
  _worker,_state,_proc => math_asin(num) Ok(ReturnValue::Value(Value::new_double(num.to_number().asin())))
);
native_fn!(
  _worker,_state,_proc => math_acos(num) Ok(ReturnValue::Value(Value::new_double(num.to_number().acos())))
);
native_fn!(
  _worker,_state,_proc => math_atan(num) Ok(ReturnValue::Value(Value::new_double(num.to_number().atan())))
);
native_fn!(
  _worker,_state,_proc => math_atan2(num1,num2) Ok(ReturnValue::Value(Value::new_double(num1.to_number().atan2(num2.to_number()))))
);

native_fn!(
  _worker,_state,_proc => math_exp_m1(num) Ok(ReturnValue::Value(Value::new_double(num.to_number().exp_m1())))
);

native_fn!(
  _worker,_state,_proc => math_ln_1p(num) Ok(ReturnValue::Value(Value::new_double(num.to_number().ln_1p())))
);

native_fn!(
    _worker,_state,_proc => math_sinh (num) Ok(ReturnValue::Value(Value::new_double(num.to_number().sinh())))
);

native_fn!(
  _worker,_state,_proc => math_cosh(num) Ok(ReturnValue::Value(Value::new_double(num.to_number().cosh())))
);

native_fn!(
  _worker,_state,_proc => math_tanh(num) Ok(ReturnValue::Value(Value::new_double(num.to_number().tanh())))
);

native_fn!(
  _worker,_state,_proc => math_asinh(num) Ok(ReturnValue::Value(Value::new_double(num.to_number().asinh())))
);
native_fn!(
  _worker,_state,_proc => math_acosh(num) Ok(ReturnValue::Value(Value::new_double(num.to_number().acosh())))
);
native_fn!(
  _worker,_state,_proc => math_atanh(num) Ok(ReturnValue::Value(Value::new_double(num.to_number().atanh())))
);

native_fn!(
    _worker,_state,_proc => math_rand(..._args) Ok(ReturnValue::Value(Value::new_double(rand::random())))
);

native_fn!(
    _worker,_state,_proc => math_rand_int(..._args) Ok(ReturnValue::Value(Value::new_int(rand::random())))
);
pub fn initialize_math(state: &RcState) {
  let mut lock = state.static_variables.lock();
  let mut cell = Cell::with_prototype(CellValue::None, state.object_prototype.as_cell());

  cell.add_attribute(
    Value::from(state.intern_string("sin".to_owned())),
    Value::from(state.allocate_native_fn(math_sin, "sin", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("cos".to_owned())),
    Value::from(state.allocate_native_fn(math_cos, "cos", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("tan".to_owned())),
    Value::from(state.allocate_native_fn(math_tan, "tan", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("asin".to_owned())),
    Value::from(state.allocate_native_fn(math_asin, "asin", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("acos".to_owned())),
    Value::from(state.allocate_native_fn(math_acos, "acos", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("atan".to_owned())),
    Value::from(state.allocate_native_fn(math_atan, "atan", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("atan2".to_owned())),
    Value::from(state.allocate_native_fn(math_atan2, "atan2", 2)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("exp_m1".to_owned())),
    Value::from(state.allocate_native_fn(math_exp_m1, "exp_m1", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("ln_1p".to_owned())),
    Value::from(state.allocate_native_fn(math_ln_1p, "ln_1p", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("sinh".to_owned())),
    Value::from(state.allocate_native_fn(math_sinh, "sinh", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("cosh".to_owned())),
    Value::from(state.allocate_native_fn(math_cosh, "cosh", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("tanh".to_owned())),
    Value::from(state.allocate_native_fn(math_tanh, "tanh", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("asinh".to_owned())),
    Value::from(state.allocate_native_fn(math_asinh, "asinh", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("acosh".to_owned())),
    Value::from(state.allocate_native_fn(math_acosh, "acosh", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("atanh".to_owned())),
    Value::from(state.allocate_native_fn(math_atanh, "atanh", 1)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("random".to_owned())),
    Value::from(state.allocate_native_fn(math_rand, "random", 0)),
  );
  cell.add_attribute(
    Value::from(state.intern_string("randomInt".to_owned())),
    Value::from(state.allocate_native_fn(math_rand_int, "randomInt", 0)),
  );
  lock.insert("Math".to_owned(), state.allocate(cell));
}
