use super::{add_generator::*, mathic::*, sub_generator::*, *};
use crate::gc::*;
use crate::value::*;
use crate::*;
use thunk_generator::*;
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
    assert_ne!(add_ic as *mut _ as usize, 0);
    let call_frame = vm.top_call_frame().unwrap();
    if let Some(profile) = add_ic
        .arith_profile
        .map(|x| unsafe { &mut *(x as *mut ArithProfile) })
    {
        profile.observe_lhs_and_rhs(op1, op2);
    }

    add_ic.generate_out_of_line(
        &call_frame.code_block.unwrap(),
        operation_value_add as *const u8,
    );
    operation_value_add(vm, op1, op2)
}
pub extern "C" fn operation_value_sub(_vm: &VM, op1: Value, op2: Value) -> Value {
    if op1.is_number() && op2.is_number() {
        let result = op1.to_number() - op2.to_number();
        if result as i32 as f64 == result {
            return Value::new_int(result as _);
        } else {
            return Value::new_double(result);
        }
    }
    // TODO: Concatenate strings,add arrays, add bigint/int64
    Value::undefined()
}

pub extern "C" fn operation_value_sub_optimize(
    vm: &VM,
    op1: Value,
    op2: Value,
    sub_ic: &mut MathIC<SubGenerator>,
) -> Value {
    let call_frame = vm.top_call_frame().unwrap();
    if let Some(profile) = sub_ic
        .arith_profile
        .map(|x| unsafe { &mut *(x as *mut ArithProfile) })
    {
        profile.observe_lhs_and_rhs(op1, op2);
    }
    sub_ic.generate_out_of_line(
        &call_frame.code_block.unwrap(),
        operation_value_sub as *const u8,
    );
    operation_value_sub(vm, op1, op2)
}
pub extern "C" fn operation_value_mul(_vm: &VM, op1: Value, op2: Value) -> Value {
    if op1.is_number() && op2.is_number() {
        let result = op1.to_number() * op2.to_number();
        if result as i32 as f64 == result {
            return Value::new_int(result as _);
        } else {
            return Value::new_double(result);
        }
    }
    Value::undefined()
}

pub extern "C" fn operation_value_mul_optimize(
    vm: &VM,
    op1: Value,
    op2: Value,
    mul_ic: &mut MathIC<mul_generator::MulGenerator>,
) -> Value {
    let call_frame = vm.top_call_frame().unwrap();
    if let Some(profile) = mul_ic
        .arith_profile
        .map(|x| unsafe { &mut *(x as *mut ArithProfile) })
    {
        profile.observe_lhs_and_rhs(op1, op2);
    }
    mul_ic.generate_out_of_line(
        &call_frame.code_block.unwrap(),
        operation_value_sub as *const u8,
    );
    operation_value_mul(vm, op1, op2)
}

pub unsafe extern "C" fn operation_link_call(
    callee_frame: *mut CallFrame,
    vm: &VM,
) -> SlowPathReturn {
    return SlowPathReturn::encode(0, 0);
}

pub extern "C" fn operation_compare_eq(x: Value, y: Value) -> bool {
    if !x.is_cell() && !y.is_cell() {
        if x.is_number() && y.is_number() {
            return x.to_number() == y.to_number();
        }
        return x == y;
    }
    let x = x.as_cell();
    let y = y.as_cell();
    if x.is_string() && y.is_string() {
        let x = x.cast::<WaffleString>();
        let y = y.cast::<WaffleString>();
        if x.len() != y.len() {
            return false;
        }
        if x.len() == 0 && y.len() == 0 {
            return true;
        }
        debug_assert!(x.len() == y.len());
        for i in 0..x.len() {
            let c1 = x.get_at(i);
            let c2 = y.get_at(i);
            if c1 != c2 {
                return false;
            }
        }
        return true;
    }
    x.ptr == y.ptr
}
pub extern "C" fn operation_compare_neq(x: Value, y: Value) -> bool {
    !operation_compare_eq(x, y)
}
