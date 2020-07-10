use super::{add_generator::*, mathic::*, *};
use crate::value::*;
use crate::*;
pub extern "C" fn operation_value_add(_vm: &VM, op1: Value, op2: Value) -> Value {
    if op1.is_number() && op2.is_number() {
        let result = op1.to_number() + op2.to_number();
        if result as i32 as f64 == result {
            return Value::new_int(result as _);
        } else {
            return Value::new_double(result);
        }
    }
    // TODO: Concatenate strings,add arrays, add bigint/int64
    Value::undefined()
}

pub extern "C" fn operation_value_add_optimize(
    vm: &VM,
    op1: Value,
    op2: Value,
    add_ic: &mut MathIC<AddGenerator>,
) -> Value {
    println!("Slow op");
    let call_frame = vm.top_call_frame().unwrap();
    if let Some(profile) = add_ic
        .arith_profile
        .map(|x| unsafe { &mut *(x as *mut ArithProfile) })
    {
        profile.observe_lhs_and_rhs(op1, op2);
    }
    add_ic.generate_out_of_line(
        call_frame.code_block_ref().unwrap(),
        operation_value_add as *const u8,
    );
    operation_value_add(vm, op1, op2)
}
