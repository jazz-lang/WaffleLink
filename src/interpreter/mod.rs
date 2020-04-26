use crate::bytecode::op::*;
use crate::common::ptr::*;
use crate::runtime::cell::*;
use crate::runtime::frame::*;
use crate::runtime::function::*;
use crate::runtime::process::*;
use crate::runtime::symbol::Symbol;
use crate::runtime::value::*;

pub mod operations;

type IFn = extern "C" fn(Frame, Ptr<Cell>, Ptr<u8>) -> Result<Value, Value>;

#[rustfmt::skip]
#[no_mangle]
static OP_TABLE: [IFn; 56] = [
    op_star, // 0
    op_ldar, // 1
    op_mov, // 2
    op_add, // 3
    op_sub, // 4
    op_div,
    op_mod,
    op_mul,
    op_ushr,
    op_shr,
    op_shl,
    op_bor,
    op_band,
    op_bxor,
    op_ldaundefined,
    op_lda_int,
    op_lda_true,
    op_lda_false,
    op_ldak,
    op_ldanull,
    op_ldaglobal,
    op_staglobal,
    op_lda_by_id,
    op_sta_by_id,
    op_lda_by_val,
    op_sta_by_val,
    op_lda_by_idx,
    op_sta_by_idx,
    op_lda_own_property,
    op_sta_own_property,
    op_lda_proto_property,
    op_sta_own_property,
    op_lda_chain_property,
    op_sta_own_property,
    op_lda_own_id,
    op_sta_own_id,
    op_lda_proto_id,
    op_sta_own_id,
    op_lda_slow_by_id,
    op_sta_slow_by_id,
    op_lda_slow_by_idx,
    op_sta_slow_by_idx,
    op_lda_slow_by_idx,
    op_sta_slow_by_idx,
    op_push_acc,
    op_pop_acc,
    op_push_reg,
    op_pop_reg,
    op_lda_this,
    op_call,
    op_throw,
    op_catch_setup,
    op_loop_hint,
    op_brc,
    op_br,
    op_return,
];

pub fn interpret_with_error_handling(mut frame: Frame, start_block: usize) -> Value {
    let function = frame.func.func_value_unchecked_mut();
    match &function.code {
        FunctionCode::Bytecode(bbs) => {
            let vpc = Ptr {
                raw: bbs[start_block].code.as_ptr() as *mut u8,
            };

            frame.exit_on_return = true;
            let f = frame.func;
            match dispatch(frame, f, vpc) {
                Err(e) => {
                    let mut catch_frame = None;
                    let ld = local_data();
                    while let Some(frame) = ld.frames.pop() {
                        if frame.try_catch.is_empty() {
                            continue;
                        } else {
                            catch_frame = Some(frame);
                            break;
                        }
                    }
                    if let Some(mut frame) = catch_frame {
                        let block = frame.try_catch.pop().unwrap();
                        frame.rax = e;
                        interpret_with_error_handling(frame, block as usize)
                    } else {
                        let mut stacktrace = String::new();

                        for (i, frame) in local_data().frames.iter_mut().enumerate() {
                            stacktrace.push_str(&format!(
                                "{}: {}\n",
                                i,
                                frame.func.func_value_unchecked_mut().name.to_string()
                            ));
                        }
                        eprintln!("Stack trace: {}", stacktrace);
                        eprintln!("Error occurred: {}", e.to_string());
                        std::process::exit(1);
                    }
                }
                Ok(res) => {
                    return res;
                }
            }
        }
        _ => unreachable!(), // this case should be handled before interpreting.
    }
}

global_asm!(
    "dispatch_asm:
        movzbl  (%rdi), %eax
        movq OP_TABLE(%rip),%rcx
        jmpq *(%rcx,%rax,8)

    "
);

#[inline(always)]
#[no_mangle]
pub extern "C" fn dispatch(_frame: Frame, _func: Ptr<Cell>, _vpc: Ptr<u8>) -> Result<Value, Value> {
    let op = *_vpc.offset(0).get();

    unsafe { OP_TABLE.get_unchecked(op as usize)(_frame, _func, _vpc) }
}

#[inline]
pub extern "C" fn op_star(frame: Frame, func: Ptr<Cell>, vpc: Ptr<u8>) -> Result<Value, Value> {
    let r = *vpc.offset(1).get();
    let rax = frame.rax;
    *frame.r(r as usize) = rax;
    dispatch(frame, func, vpc.offset(2))
}

#[inline]
pub extern "C" fn op_ldar(mut frame: Frame, func: Ptr<Cell>, vpc: Ptr<u8>) -> Result<Value, Value> {
    let r = *vpc.offset(1).get();
    let value = *frame.r(r as usize);
    frame.rax = value;
    dispatch(frame, func, vpc.offset(2))
}

#[inline]
pub extern "C" fn op_mov(frame: Frame, func: Ptr<Cell>, vpc: Ptr<u8>) -> Result<Value, Value> {
    let r = *vpc.offset(1).get();
    let r2 = *vpc.offset(2).get();
    let value = *frame.r(r2 as _);
    *frame.r(r as _) = value;
    dispatch(frame, func, vpc.offset(3))
}
#[naked]
#[inline]
pub extern "C" fn op_add(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    mut vpc: Ptr<u8>,
) -> Result<Value, Value> {
    unsafe {
        let r = *vpc.offset(1).get();
        let feedback_idx = *vpc.offset(2).cast::<u32>().get();
        let acc = frame.rax;
        let val = *frame.r(r as usize);
        vpc = vpc.offset(6);
        if acc.is_int32() && val.is_int32() {
            *func
                .func_value_unchecked_mut()
                .feedback_vector
                .get_unchecked_mut(feedback_idx as usize) =
                FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
                    Type::Int32,
                    Type::Int32,
                    Type::Int32,
                ]));
            frame.rax = Value::new_int(acc.as_int32().wrapping_add(val.as_int32()));

            return dispatch(frame, func, vpc);
        } else {
            frame.rax = op_add_slow(acc, val, &mut frame);
            let rax_ty = frame.rax.primitive_ty();
            *func
                .func_value_unchecked_mut()
                .feedback_vector
                .get_unchecked_mut(feedback_idx as usize) =
                FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
                    acc.primitive_ty(),
                    val.primitive_ty(),
                    rax_ty,
                ]));
            dispatch(frame, func, vpc)
        }
    }
}

pub fn op_add_slow(x: Value, y: Value, f: &mut Frame) -> Value {
    if x.is_number() && y.is_number() {
        Value::new_double(x.to_number() + y.to_number())
    } else if x.is_null_or_undefined() || y.is_null_or_undefined() {
        Value::from(VTag::Undefined)
    } else {
        local_data().allocate_string(format!("{}{}", x.to_string(), y.to_string()), f)
    }
}
#[naked]
#[inline]
pub extern "C" fn op_sub(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    mut vpc: Ptr<u8>,
) -> Result<Value, Value> {
    unsafe {
        let r = *vpc.offset(1).get();
        let feedback_idx = *vpc.offset(2).cast::<u32>().get();
        let feedback = func
            .func_value_unchecked_mut()
            .feedback_vector
            .get_unchecked_mut(feedback_idx as usize);
        let acc = frame.rax;
        let val = *frame.r(r as usize);
        vpc = vpc.offset(6);
        if acc.is_int32() && val.is_int32() {
            frame.rax = Value::new_int(acc.as_int32() - val.as_int32());
        } else if acc.is_number() && val.is_number() {
            frame.rax = Value::new_double(acc.to_number() - val.to_number());
        } else {
            frame.rax = op_sub_slow(acc, val);
        }
        *feedback = FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
            acc.primitive_ty(),
            val.primitive_ty(),
            frame.rax.primitive_ty(),
        ]));
        dispatch(frame, func, vpc)
    }
}

fn op_sub_slow(_x: Value, _y: Value) -> Value {
    Value::new_double(std::f64::NAN)
}

#[inline]
pub extern "C" fn op_div(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    mut vpc: Ptr<u8>,
) -> Result<Value, Value> {
    unsafe {
        let r = *vpc.offset(1).get();
        let feedback_idx = *vpc.offset(2).cast::<u32>().get();
        let feedback = func
            .func_value_unchecked_mut()
            .feedback_vector
            .get_unchecked_mut(feedback_idx as usize);
        let acc = frame.rax;
        let val = *frame.r(r as usize);
        vpc = vpc.offset(6);
        if acc.is_int32() && val.is_int32() {
            if val.as_int32() == 0 {
                frame.rax = Value::new_double(std::f64::NAN);
            } else {
                frame.rax = Value::new_int(acc.as_int32() / val.as_int32());
            }
        } else if acc.is_number() && val.is_number() {
            if val.as_int32() != 0 {
                frame.rax = Value::new_double(acc.to_number() / val.to_number());
            } else {
                frame.rax = Value::new_double(std::f64::NAN);
            }
        } else {
            frame.rax = Value::new_double(std::f64::NAN);
        }
        *feedback = FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
            acc.primitive_ty(),
            val.primitive_ty(),
            frame.rax.primitive_ty(),
        ]));
        dispatch(frame, func, vpc)
    }
}

#[inline]
pub extern "C" fn op_mul(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    mut vpc: Ptr<u8>,
) -> Result<Value, Value> {
    unsafe {
        let r = *vpc.offset(1).get();
        let feedback_idx = *vpc.offset(2).cast::<u32>().get();
        let feedback = func
            .func_value_unchecked_mut()
            .feedback_vector
            .get_unchecked_mut(feedback_idx as usize);
        let acc = frame.rax;
        let val = *frame.r(r as usize);
        vpc = vpc.offset(6);
        if acc.is_int32() && val.is_int32() {
            frame.rax = Value::new_int(acc.as_int32() * val.as_int32());
        } else if acc.is_number() && val.is_number() {
            frame.rax = Value::new_double(acc.to_number() * val.to_number());
        } else {
            frame.rax = Value::new_double(std::f64::NAN);
        }
        *feedback = FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
            acc.primitive_ty(),
            val.primitive_ty(),
            frame.rax.primitive_ty(),
        ]));
        dispatch(frame, func, vpc)
    }
}

#[inline]
pub extern "C" fn op_mod(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    mut vpc: Ptr<u8>,
) -> Result<Value, Value> {
    unsafe {
        let r = *vpc.offset(1).get();
        let feedback_idx = *vpc.offset(2).cast::<u32>().get();
        let feedback = func
            .func_value_unchecked_mut()
            .feedback_vector
            .get_unchecked_mut(feedback_idx as usize);
        let acc = frame.rax;
        let val = *frame.r(r as usize);
        vpc = vpc.offset(6);
        if acc.is_int32() && val.is_int32() {
            frame.rax = Value::new_int(acc.as_int32() % val.as_int32());
        } else if acc.is_number() && val.is_number() {
            frame.rax = Value::new_double(acc.to_number() % val.to_number());
        } else {
            frame.rax = Value::new_double(std::f64::NAN);
        }
        *feedback = FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
            acc.primitive_ty(),
            val.primitive_ty(),
            frame.rax.primitive_ty(),
        ]));
        dispatch(frame, func, vpc)
    }
}

#[inline]
pub extern "C" fn op_ushr(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    mut vpc: Ptr<u8>,
) -> Result<Value, Value> {
    unsafe {
        let r = *vpc.offset(1).get();
        let feedback_idx = *vpc.offset(2).cast::<u32>().get();
        let feedback = func
            .func_value_unchecked_mut()
            .feedback_vector
            .get_unchecked_mut(feedback_idx as usize);
        let acc = frame.rax;
        let val = *frame.r(r as usize);
        vpc = vpc.offset(6);
        if acc.is_int32() && val.is_int32() {
            frame.rax = Value::new_int(acc.as_int32() >> val.as_int32());
        } else if acc.is_number() && val.is_number() {
            frame.rax = Value::new_int(
                ((acc.to_number().floor() as u32) >> val.to_number().floor() as u32) as i32,
            );
        } else {
            frame.rax = Value::new_double(std::f64::NAN);
        }
        *feedback = FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
            acc.primitive_ty(),
            val.primitive_ty(),
            frame.rax.primitive_ty(),
        ]));
        dispatch(frame, func, vpc)
    }
}

#[inline]
pub extern "C" fn op_shr(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    mut vpc: Ptr<u8>,
) -> Result<Value, Value> {
    unsafe {
        let r = *vpc.offset(1).get();
        let feedback_idx = *vpc.offset(2).cast::<u32>().get();
        let feedback = func
            .func_value_unchecked_mut()
            .feedback_vector
            .get_unchecked_mut(feedback_idx as usize);
        let acc = frame.rax;
        let val = *frame.r(r as usize);
        vpc = vpc.offset(6);
        if acc.is_int32() && val.is_int32() {
            frame.rax = Value::new_int(acc.as_int32() >> val.as_int32());
        } else if acc.is_number() && val.is_number() {
            frame.rax = Value::new_int(
                ((acc.to_number().floor() as i32) >> val.to_number().floor() as i32) as i32,
            );
        } else {
            frame.rax = Value::new_double(std::f64::NAN);
        }
        *feedback = FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
            acc.primitive_ty(),
            val.primitive_ty(),
            frame.rax.primitive_ty(),
        ]));
        dispatch(frame, func, vpc)
    }
}

#[inline]
pub extern "C" fn op_shl(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    mut vpc: Ptr<u8>,
) -> Result<Value, Value> {
    unsafe {
        let r = *vpc.offset(1).get();
        let feedback_idx = *vpc.offset(2).cast::<u32>().get();
        let feedback = func
            .func_value_unchecked_mut()
            .feedback_vector
            .get_unchecked_mut(feedback_idx as usize);
        let acc = frame.rax;
        let val = *frame.r(r as usize);
        vpc = vpc.offset(6);
        if acc.is_int32() && val.is_int32() {
            frame.rax = Value::new_int(acc.as_int32() << val.as_int32());
        } else if acc.is_number() && val.is_number() {
            frame.rax = Value::new_int(
                ((acc.to_number().floor() as i32) << val.to_number().floor() as i32) as i32,
            );
        } else {
            frame.rax = Value::new_double(std::f64::NAN);
        }
        *feedback = FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
            acc.primitive_ty(),
            val.primitive_ty(),
            frame.rax.primitive_ty(),
        ]));
        dispatch(frame, func, vpc)
    }
}

#[inline]
pub extern "C" fn op_bor(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    mut vpc: Ptr<u8>,
) -> Result<Value, Value> {
    unsafe {
        let r = *vpc.offset(1).get();
        let feedback_idx = *vpc.offset(2).cast::<u32>().get();
        let feedback = func
            .func_value_unchecked_mut()
            .feedback_vector
            .get_unchecked_mut(feedback_idx as usize);
        let acc = frame.rax;
        let val = *frame.r(r as usize);
        vpc = vpc.offset(6);
        if acc.is_int32() && val.is_int32() {
            frame.rax = Value::new_int(acc.as_int32() | val.as_int32());
        } else if acc.is_number() && val.is_number() {
            frame.rax = Value::new_int(
                ((acc.to_number().floor() as i32) | val.to_number().floor() as i32) as i32,
            );
        } else {
            frame.rax = Value::new_double(std::f64::NAN);
        }
        *feedback = FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
            acc.primitive_ty(),
            val.primitive_ty(),
            frame.rax.primitive_ty(),
        ]));
        dispatch(frame, func, vpc)
    }
}

#[inline]
pub extern "C" fn op_band(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    mut vpc: Ptr<u8>,
) -> Result<Value, Value> {
    unsafe {
        let r = *vpc.offset(1).get();
        let feedback_idx = *vpc.offset(2).cast::<u32>().get();
        let feedback = func
            .func_value_unchecked_mut()
            .feedback_vector
            .get_unchecked_mut(feedback_idx as usize);
        let acc = frame.rax;
        let val = *frame.r(r as usize);
        vpc = vpc.offset(6);
        if acc.is_int32() && val.is_int32() {
            frame.rax = Value::new_int(acc.as_int32() & val.as_int32());
        } else if acc.is_number() && val.is_number() {
            frame.rax = Value::new_int(
                ((acc.to_number().floor() as i32) & val.to_number().floor() as i32) as i32,
            );
        } else {
            frame.rax = Value::new_double(std::f64::NAN);
        }
        *feedback = FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
            acc.primitive_ty(),
            val.primitive_ty(),
            frame.rax.primitive_ty(),
        ]));
        dispatch(frame, func, vpc)
    }
}

#[inline]
pub extern "C" fn op_bxor(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    mut vpc: Ptr<u8>,
) -> Result<Value, Value> {
    unsafe {
        let r = *vpc.offset(1).get();
        let feedback_idx = *vpc.offset(2).cast::<u32>().get();
        let feedback = func
            .func_value_unchecked_mut()
            .feedback_vector
            .get_unchecked_mut(feedback_idx as usize);
        let acc = frame.rax;
        let val = *frame.r(r as usize);
        vpc = vpc.offset(6);
        if acc.is_int32() && val.is_int32() {
            frame.rax = Value::new_int(acc.as_int32() ^ val.as_int32());
        } else if acc.is_number() && val.is_number() {
            frame.rax = Value::new_int(
                ((acc.to_number().floor() as i32) ^ val.to_number().floor() as i32) as i32,
            );
        } else {
            frame.rax = Value::new_double(std::f64::NAN);
        }
        *feedback = FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
            acc.primitive_ty(),
            val.primitive_ty(),
            frame.rax.primitive_ty(),
        ]));
        dispatch(frame, func, vpc)
    }
}

#[inline]
pub extern "C" fn op_ldaundefined(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    frame.rax = Value::from(VTag::Undefined);
    dispatch(frame, func, vpc.offset(1))
}

#[inline]
pub extern "C" fn op_lda_int(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let i = *vpc.offset(1).cast::<i32>();
    frame.rax = Value::new_int(i);
    dispatch(frame, func, vpc.offset(5))
}

#[inline]
pub extern "C" fn op_lda_true(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    frame.rax = Value::from(VTag::True);
    dispatch(frame, func, vpc.offset(1))
}

#[inline]
pub extern "C" fn op_lda_false(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    frame.rax = Value::from(VTag::False);
    dispatch(frame, func, vpc.offset(1))
}

#[inline]
pub extern "C" fn op_ldak(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let constant = *vpc.offset(1).cast::<u32>();
    unsafe {
        frame.rax = *func
            .func_value_unchecked_mut()
            .constants
            .get_unchecked(constant as usize);
    }
    dispatch(frame, func, vpc.offset(5))
}

#[inline]
pub extern "C" fn op_ldanull(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    frame.rax = Value::from(VTag::Null);
    dispatch(frame, func, vpc.offset(5))
}

#[inline]
pub extern "C" fn op_ldaglobal(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let constant = *vpc.offset(1).cast::<u32>();
    let key = unsafe {
        let key = *func
            .func_value_unchecked_mut()
            .constants
            .get_unchecked(constant as usize);
        key
    };
    frame.rax = local_data()
        .globals
        .get(&Symbol::new_value(key))
        .copied()
        .unwrap_or(Value::from(VTag::Undefined));
    dispatch(frame, func, vpc.offset(5))
}

#[inline]
pub extern "C" fn op_staglobal(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let constant = *vpc.offset(1).cast::<u32>();
    let key = unsafe {
        let key = *func
            .func_value_unchecked_mut()
            .constants
            .get_unchecked(constant as usize);
        key
    };

    local_data()
        .globals
        .insert(Symbol::new_value(key), frame.rax);
    dispatch(frame, func, vpc.offset(5))
}

#[inline]
pub extern "C" fn op_lda_by_id(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    operations::lda_by_id_impl(&mut frame, func, vpc);
    dispatch(frame, func, vpc.offset(10))
}

#[inline]
pub extern "C" fn op_lda_by_val(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2);
    let mut base = *frame.r(base_reg as usize);
    let sym = Symbol::new_value(*frame.r(key as usize));
    let mut slot = Slot::new();
    base.lookup(sym, &mut slot);
    frame.rax = slot.value();
    dispatch(frame, func, vpc.offset(3))
}

#[inline]
pub extern "C" fn op_sta_by_val(
    frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2);
    let mut base = *frame.r(base_reg as usize);
    let sym = Symbol::new_value(*frame.r(key as usize));
    let mut slot = Slot::new();
    base.insert(sym, frame.rax, &mut slot);
    dispatch(frame, func, vpc.offset(3))
}

#[inline]
pub extern "C" fn op_lda_by_idx(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    operations::lda_by_idx_impl(&mut frame, func, vpc);
    dispatch(frame, func, vpc.offset(10))
}

#[inline]
pub extern "C" fn op_sta_by_idx(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    operations::sta_by_idx_impl(&mut frame, func, vpc);
    dispatch(frame, func, vpc.offset(10))
}

#[inline]
pub extern "C" fn op_sta_by_id(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    operations::sta_by_id_impl(&mut frame, func, vpc);
    dispatch(frame, func, vpc.offset(10))
}

#[inline]
pub extern "C" fn op_lda_own_property(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = unsafe {
        let key = *func
            .func_value_unchecked_mut()
            .constants
            .get_unchecked(key as usize);
        key
    };
    let mut slot = Slot::new();
    let mut base = *frame.r(base_reg as usize);
    if !base.is_cell() {
        base.lookup(Symbol::new_value(key), &mut slot);
        frame.rax = slot.value(); // this is undefined or property.
        return dispatch(frame, func, vpc.offset(10));
    }
    let cell = base.as_cell();
    let cache = vpc.offset(6).cast::<u32>();
    let f = func.func_value_unchecked_mut();
    let feedback = unsafe { f.feedback_vector.get_unchecked_mut(*cache as usize) };
    if let FeedBack::Cache(map, offset, misses) = feedback {
        if map.raw == cell.attributes.raw {
            // cache hit
            frame.rax = cell.direct(*offset);
        } else {
            *misses += 1;
            // cache miss
            //
            // invoke slow path, slow path will try to cache again.
            operations::lda_by_id_impl(&mut frame, func, vpc)
        }
    }
    return dispatch(frame, func, vpc.offset(10));
}

#[inline]
pub extern "C" fn op_lda_proto_property(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = unsafe {
        let key = *func
            .func_value_unchecked_mut()
            .constants
            .get_unchecked(key as usize);
        key
    };
    let mut slot = Slot::new();
    let mut base = *frame.r(base_reg as usize);
    if !base.is_cell() {
        base.lookup(Symbol::new_value(key), &mut slot);
        frame.rax = slot.value(); // this is undefined or property.
        return dispatch(frame, func, vpc.offset(10));
    }
    let cell = base.as_cell();
    let cache = vpc.offset(6).cast::<u32>();
    let f = func.func_value_unchecked_mut();
    let feedback = unsafe { f.feedback_vector.get_unchecked_mut(*cache as usize) };
    if let Some(cell) = cell.prototype {
        if let FeedBack::Cache(map, offset, misses) = feedback {
            if map.raw == cell.attributes.raw {
                // cache hit
                frame.rax = cell.direct(*offset);
            } else {
                *misses += 1;
                // cache miss
                //
                // invoke slow path, slow path will try to cache again.
                operations::lda_by_id_impl(&mut frame, func, vpc)
            }
        }
    } else {
        // cache miss
        operations::lda_by_id_impl(&mut frame, func, vpc)
    }
    return dispatch(frame, func, vpc.offset(10));
}

#[inline]
pub extern "C" fn op_lda_chain_property(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = unsafe {
        let key = *func
            .func_value_unchecked_mut()
            .constants
            .get_unchecked(key as usize);
        key
    };
    let mut slot = Slot::new();
    let mut base = *frame.r(base_reg as usize);
    if !base.is_cell() {
        // TODO: Does this case count as cache miss?
        base.lookup(Symbol::new_value(key), &mut slot);
        frame.rax = slot.value(); // this is undefined or property.
        return dispatch(frame, func, vpc.offset(10));
    }
    let mut cell = Some(base.as_cell());
    let cache = vpc.offset(6).cast::<u32>();
    let f = func.func_value_unchecked_mut();
    let feedback = unsafe { f.feedback_vector.get_unchecked_mut(*cache as usize) };
    if let FeedBack::Cache(map, offset, misses) = feedback {
        while let Some(proto) = cell {
            if map.raw == proto.attributes.raw {
                // cache hit
                frame.rax = proto.direct(*offset);
                return dispatch(frame, func, vpc.offset(10));
            } else {
                cell = proto.prototype;
            }
        }
        *misses += 1;
    }

    // cache miss
    operations::lda_by_id_impl(&mut frame, func, vpc);

    return dispatch(frame, func, vpc.offset(10));
}

#[inline]
pub extern "C" fn op_sta_own_property(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = unsafe {
        let key = *func
            .func_value_unchecked_mut()
            .constants
            .get_unchecked(key as usize);
        key
    };
    let mut slot = Slot::new();
    let mut base = *frame.r(base_reg as usize);

    if !base.is_cell() {
        /*base.lookup(Symbol::new_value(key), &mut slot);
        frame.rax = slot.value(); // this is undefined or property.*/
        base.insert(Symbol::new_value(key), frame.rax, &mut slot);
        return dispatch(frame, func, vpc.offset(10));
    }
    let mut cell = base.as_cell();
    let cache = vpc.offset(6).cast::<u32>();
    let f = func.func_value_unchecked_mut();
    let feedback = unsafe { f.feedback_vector.get_unchecked_mut(*cache as usize) };
    if let FeedBack::Cache(map, offset, misses) = feedback {
        if map.raw == cell.attributes.raw {
            // cache hit
            cell.store_direct(*offset, frame.rax);
        } else {
            *misses += 1;
            // cache miss
            //
            // invoke slow path, slow path will try to cache again.
            operations::sta_by_id_impl(&mut frame, func, vpc)
        }
    } else {
        operations::sta_by_id_impl(&mut frame, func, vpc);
    }
    return dispatch(frame, func, vpc.offset(10));
}

#[inline]
pub extern "C" fn op_sta_own_id(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = Value::new_int(key as i32);
    let mut slot = Slot::new();
    let mut base = *frame.r(base_reg as usize);

    if !base.is_cell() {
        /*base.lookup(Symbol::new_value(key), &mut slot);
        frame.rax = slot.value(); // this is undefined or property.*/
        base.insert(Symbol::new_value(key), frame.rax, &mut slot);
        return dispatch(frame, func, vpc.offset(10));
    }
    let mut cell = base.as_cell();
    let cache = vpc.offset(6).cast::<u32>();
    let f = func.func_value_unchecked_mut();
    let feedback = unsafe { f.feedback_vector.get_unchecked_mut(*cache as usize) };
    if let FeedBack::Cache(map, offset, misses) = feedback {
        if map.raw == cell.attributes.raw {
            // cache hit
            cell.store_direct(*offset, frame.rax);
        } else {
            *misses += 1;
            // cache miss
            //
            // invoke slow path, slow path will try to cache again.
            operations::sta_by_id_impl(&mut frame, func, vpc)
        }
    } else {
        operations::sta_by_id_impl(&mut frame, func, vpc);
    }
    return dispatch(frame, func, vpc.offset(10));
}
#[inline]
pub extern "C" fn op_lda_own_id(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = Value::new_int(key as i32);
    let mut slot = Slot::new();
    let mut base = *frame.r(base_reg as usize);

    if !base.is_cell() {
        /*base.lookup(Symbol::new_value(key), &mut slot);
        frame.rax = slot.value(); // this is undefined or property.*/
        base.insert(Symbol::new_value(key), frame.rax, &mut slot);
        return dispatch(frame, func, vpc.offset(10));
    }
    let cell = base.as_cell();
    let cache = vpc.offset(6).cast::<u32>();
    let f = func.func_value_unchecked_mut();
    let feedback = unsafe { f.feedback_vector.get_unchecked_mut(*cache as usize) };
    if let FeedBack::Cache(map, offset, misses) = feedback {
        if map.raw == cell.attributes.raw {
            // cache hit
            frame.rax = cell.direct(*offset);
        //cell.store_direct(*offset, frame.rax);
        } else {
            *misses += 1;
            // cache miss
            //
            // invoke slow path, slow path will try to cache again.
            operations::lda_by_id_impl(&mut frame, func, vpc)
        }
    } else {
        operations::lda_by_id_impl(&mut frame, func, vpc);
    }
    return dispatch(frame, func, vpc.offset(10));
}

#[inline]
pub extern "C" fn op_lda_proto_id(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = Value::new_int(key as i32);
    let mut slot = Slot::new();
    let mut base = *frame.r(base_reg as usize);

    if !base.is_cell() {
        /*base.lookup(Symbol::new_value(key), &mut slot);
        frame.rax = slot.value(); // this is undefined or property.*/
        base.insert(Symbol::new_value(key), frame.rax, &mut slot);
        return dispatch(frame, func, vpc.offset(10));
    }
    let cell = base.as_cell();
    let cache = vpc.offset(6).cast::<u32>();
    let f = func.func_value_unchecked_mut();
    let feedback = unsafe { f.feedback_vector.get_unchecked_mut(*cache as usize) };
    if let FeedBack::Cache(map, offset, misses) = feedback {
        if let Some(proto) = cell.prototype {
            if map.raw == proto.attributes.raw {
                // cache hit
                frame.rax = proto.direct(*offset);
            //cell.store_direct(*offset, frame.rax);
            } else {
                *misses += 1;
                operations::lda_by_idx_impl(&mut frame, func, vpc)
            }
        } else {
            *misses += 1;
            // cache miss
            //
            // invoke slow path, slow path will try to cache again.
            operations::lda_by_idx_impl(&mut frame, func, vpc)
        }
    } else {
        operations::lda_by_idx_impl(&mut frame, func, vpc);
    }
    return dispatch(frame, func, vpc.offset(10));
}

#[inline]
pub extern "C" fn op_lda_slow_by_id(
    mut frame: Frame,
    mut func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = unsafe {
        let key = *func
            .func_value_unchecked_mut()
            .constants
            .get_unchecked(key as usize);
        key
    };
    let mut base = *frame.r(base_reg as usize);
    let mut slot = Slot::new();
    base.lookup(Symbol::new_value(key), &mut slot);
    frame.rax = slot.value();

    dispatch(frame, func, vpc.offset(10))
}
#[inline]
pub extern "C" fn op_sta_slow_by_id(
    frame: Frame,
    mut func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = unsafe {
        let key = *func
            .func_value_unchecked_mut()
            .constants
            .get_unchecked(key as usize);
        key
    };
    let mut base = *frame.r(base_reg as usize);
    let mut slot = Slot::new();
    base.insert(Symbol::new_value(key), frame.rax, &mut slot);

    dispatch(frame, func, vpc.offset(10))
}

#[inline]
pub extern "C" fn op_lda_slow_by_idx(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = Value::new_int(key as i32);
    let mut base = *frame.r(base_reg as usize);
    let mut slot = Slot::new();
    base.lookup(Symbol::new_value(key), &mut slot);
    frame.rax = slot.value();

    dispatch(frame, func, vpc.offset(10))
}
#[inline]
pub extern "C" fn op_sta_slow_by_idx(
    frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = Value::new_int(key as i32);
    let mut base = *frame.r(base_reg as usize);
    let mut slot = Slot::new();
    base.insert(Symbol::new_value(key), frame.rax, &mut slot);

    dispatch(frame, func, vpc.offset(10))
}

#[inline]
pub extern "C" fn op_push_acc(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let acc = frame.rax;
    frame.stack.push(acc);
    dispatch(frame, func, vpc.offset(1))
}

#[inline]
pub extern "C" fn op_pop_acc(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let value = frame.stack.pop().unwrap_or(Value::from(VTag::Undefined));
    frame.rax = value;
    dispatch(frame, func, vpc.offset(1))
}

#[inline]
pub extern "C" fn op_push_reg(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let reg = *vpc.offset(1);
    let value = *frame.r(reg as usize);
    frame.stack.push(value);
    dispatch(frame, func, vpc.offset(2))
}

#[inline]
pub extern "C" fn op_pop_reg(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let reg = *vpc.offset(1);
    let value = frame.stack.pop().unwrap_or(Value::from(VTag::Undefined));
    *frame.r(reg as usize) = value;
    dispatch(frame, func, vpc.offset(2))
}

#[inline]
pub extern "C" fn op_lda_this(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let this = frame.this;
    frame.rax = this;
    dispatch(frame, func, vpc.offset(1))
}

#[inline]
pub extern "C" fn op_call(mut frame: Frame, func: Ptr<Cell>, vpc: Ptr<u8>) -> Result<Value, Value> {
    let function = *vpc.offset(1);
    let argc = *vpc.offset(2).cast::<u32>();
    let this = frame.rax;
    let function = *frame.r(function as _);
    if !function.is_cell() {
        return Err(local_data().allocate_string(
            format!("{} is not a function", function.to_string()),
            &mut frame,
        ));
    }

    let mut cell = function.as_cell();
    let mut arguments = vec![];
    for _ in 0..argc {
        arguments.push(frame.pop());
    }
    if cell.is_function() {
        let f = cell.func_value_unchecked_mut();
        // TODO: Compile or invoke JITed code.
        #[cfg(feature = "jit")]
        {
            if f.can_jit {
                f.threshold += 50;
                match f.threshold {
                    // Simple JIT
                    500 => {}
                    // Easy JIT
                    10000 => {}
                    // Full JIT
                    100000 => {}
                    _ => (),
                }
            }
        }
        let (mut new_frame, code) = match &f.code {
            FunctionCode::Bytecode(code) => {
                let mut new_frame = Frame::new(this, f.module);
                new_frame.stack = arguments.clone();
                new_frame.arguments = arguments;

                (new_frame, code)
            }
            FunctionCode::Native(fun) => {
                let mut new_frame = Frame::native_frame(this, arguments, f.module);
                frame.rax = fun(&mut new_frame, cell, vpc)?;
                return dispatch(frame, func, vpc.offset(6));
            }
        };
        let new_vpc = Ptr {
            raw: code[0].code.as_ptr() as *mut u8,
        };
        frame.ip = vpc.raw as usize;
        local_data().frames.push(frame);
        return dispatch(new_frame, cell, new_vpc);
    } else {
        return Err(local_data().allocate_string(
            format!("{} is not a function", function.to_string()),
            &mut frame,
        ));
    }
}

#[inline]
pub extern "C" fn op_throw(frame: Frame, _: Ptr<Cell>, _: Ptr<u8>) -> Result<Value, Value> {
    let exception = frame.rax;

    local_data().frames.push(frame);
    Err(exception)
}

#[inline]
pub extern "C" fn op_catch_setup(
    mut frame: Frame,
    func: Ptr<Cell>,
    vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let catch_block = *vpc.offset(1).cast::<u32>();
    frame.try_catch.push(catch_block);
    dispatch(frame, func, vpc.offset(5))
}

// TODO: increment function hotness and loop hotness and jit compile loop if needed.
#[inline]
pub extern "C" fn op_loop_hint(
    _frame: Frame,
    _func: Ptr<Cell>,
    _vpc: Ptr<u8>,
) -> Result<Value, Value> {
    unimplemented!()
}

#[inline]
pub extern "C" fn op_brc(
    frame: Frame,
    mut func: Ptr<Cell>,
    mut vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let condition = frame.rax.to_boolean();
    let if_true = *vpc.offset(1).cast::<u32>();
    let if_false = *vpc.offset(5).cast::<u32>();
    if condition {
        unsafe {
            vpc = Ptr {
                raw: func
                    .func_value_unchecked_mut()
                    .get_bytecode_unchecked()
                    .get_unchecked(if_true as usize)
                    .code
                    .as_ptr() as *mut u8,
            };
        }
    } else {
        unsafe {
            vpc = Ptr {
                raw: func
                    .func_value_unchecked_mut()
                    .get_bytecode_unchecked()
                    .get_unchecked(if_false as usize)
                    .code
                    .as_ptr() as *mut u8,
            };
        }
    }
    dispatch(frame, func, vpc)
}

#[inline]
pub extern "C" fn op_br(
    frame: Frame,
    mut func: Ptr<Cell>,
    mut vpc: Ptr<u8>,
) -> Result<Value, Value> {
    let block = *vpc.offset(1).cast::<u32>();
    unsafe {
        vpc = Ptr {
            raw: func
                .func_value_unchecked_mut()
                .get_bytecode_unchecked()
                .get_unchecked(block as usize)
                .code
                .as_ptr() as *mut u8,
        };
    }
    dispatch(frame, func, vpc)
}

#[inline]
pub extern "C" fn op_return(frame: Frame, _func: Ptr<Cell>, _vpc: Ptr<u8>) -> Result<Value, Value> {
    return Ok(frame.rax);
}
