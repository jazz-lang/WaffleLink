use super::{add_generator::*, mathic::*, sub_generator::*, *};
use crate::gc::*;
use crate::value::*;
use crate::*;
use thunk_generator::*;
use virtual_register::*;
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

pub extern "C" fn operation_compare_less(x: Value, y: Value) -> bool {
    if x.is_number() && y.is_number() {
        return x.to_number() < y.to_number();
    }
    if x.is_cell() && y.is_cell() {
        let x = x.as_cell();
        let y = y.as_cell();
        if x.is_string() && y.is_string() {
            return x.cast::<WaffleString>().len() < y.cast::<WaffleString>().len();
        } else if x.is_array_ref() && y.is_array_ref() {
            return x.cast::<Array>().len() < y.cast::<Array>().len();
        }
    }

    false
}
pub extern "C" fn operation_compare_greater(x: Value, y: Value) -> bool {
    if x.is_number() && y.is_number() {
        return x.to_number() > y.to_number();
    }
    if x.is_cell() && y.is_cell() {
        let x = x.as_cell();
        let y = y.as_cell();
        if x.is_string() && y.is_string() {
            return x.cast::<WaffleString>().len() > y.cast::<WaffleString>().len();
        } else if x.is_array_ref() && y.is_array_ref() {
            return x.cast::<Array>().len() > y.cast::<Array>().len();
        }
    }

    false
}
pub extern "C" fn operation_compare_lesseq(x: Value, y: Value) -> bool {
    let xv = x;
    let yv = y;
    if x.is_number() && y.is_number() {
        return x.to_number() <= y.to_number();
    }
    if x.is_cell() && y.is_cell() {
        let x = x.as_cell();
        let y = y.as_cell();
        if x.is_string() && y.is_string() {
            return (x.cast::<WaffleString>().len() < y.cast::<WaffleString>().len())
                || operation_compare_eq(xv, yv);
        } else if x.is_array_ref() && y.is_array_ref() {
            return (x.cast::<Array>().len() < y.cast::<Array>().len()) || x.raw() == y.raw();
        }
    }

    x == y
}
pub extern "C" fn operation_compare_greaterq(x: Value, y: Value) -> bool {
    let xv = x;
    let yv = y;
    if x.is_number() && y.is_number() {
        return x.to_number() >= y.to_number();
    }
    if x.is_cell() && y.is_cell() {
        let x = x.as_cell();
        let y = y.as_cell();
        if x.is_string() && y.is_string() {
            return (x.cast::<WaffleString>().len() > y.cast::<WaffleString>().len())
                || operation_compare_eq(xv, yv);
        } else if x.is_array_ref() && y.is_array_ref() {
            return (x.cast::<Array>().len() > y.cast::<Array>().len()) || x.raw() == y.raw();
        }
    }

    x == y
}

pub extern "C" fn operation_compare_neq(x: Value, y: Value) -> bool {
    !operation_compare_eq(x, y)
}

pub extern "C" fn operation_call_func(
    cf: &mut CallFrame,
    callee: Value,
    argc: u32,
    this: Value,
) -> WaffleResult {
    let mut args = vec![];
    {
        for i in 0..argc {
            args.push(cf.get_register(VirtualRegister::new_argument(i as _)));
        }
    }

    if let Some((addr, argc, vars, cb)) = get_executable_address_for(callee) {
        let passed = args.len();
        if args.len() < argc as usize {
            while args.len() < argc as usize {
                args.push(Value::undefined());
            }
        }
        let mut call_frame = CallFrame::new(&args, vars);
        call_frame.this = this;
        call_frame.callee = callee;
        call_frame.passed_argc = passed as u32;
        call_frame.code_block = cb;
        let vm = crate::get_vm();
        vm.call_stack.push(call_frame);
        let result = addr(&mut vm.call_stack.last_mut().unwrap());
        vm.call_stack.pop().unwrap();
        return result;
    }
    get_vm().throw_exception_str("callee is not a function")
}

pub fn get_executable_address_for(
    v: Value,
) -> Option<(
    extern "C" fn(cf: &mut CallFrame) -> WaffleResult,
    u32,
    u32,
    Option<Ref<CodeBlock>>,
)> {
    if v.is_cell() && v.as_cell().is_function() {
        let cell = v.as_cell();
        let mut cell = cell.cast::<function::Function>();
        if cell.native {
            return Some(unsafe { (std::mem::transmute(cell.native_code), 0, 0, None) });
        }
        cell.code_block.exc_counter += 50;
        let args = cell.code_block.num_args;
        let vars = cell.code_block.num_vars;
        let cb = cell.code_block;
        let lock = cb.jit_data();
        if lock.executable_addr != 0 {
            let addr = lock.executable_addr;
            drop(lock);
            return unsafe { Some((std::mem::transmute(addr), args, vars, Some(cell.code_block))) };
        } else if cell.code_block.exc_counter >= crate::get_vm().jit_threshold {
            drop(lock);
            log!(
                "Trying to compile function code block at {:p}",
                cell.code_block.raw()
            );
            let mut jit = JIT::new(&cell.code_block);
            jit.compile_without_linking();
            jit.link();
            if crate::get_vm().disasm {
                jit.disasm();
            }
            let lock = cb.jit_data();
            if lock.executable_addr != 0 {
                let addr = lock.executable_addr;
                drop(lock);
                return unsafe {
                    Some((std::mem::transmute(addr), args, vars, Some(cell.code_block)))
                };
            } else {
                // woops! JIT somehow managed to fail.
                return Some((interpreter::interp_loop, args, vars, Some(cell.code_block)));
            }
        } else {
            return Some((interpreter::interp_loop, args, vars, Some(cell.code_block)));
        }
    }
    None
}
