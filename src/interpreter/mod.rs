use crate::bytecode::op::*;
use crate::heap::*;
use crate::runtime;
use runtime::cell::*;
use runtime::frame::*;
use runtime::function::*;
use runtime::process::*;
use runtime::value::*;
pub fn run(mut frame: Frame) -> Result<Value, Value> {
    loop {
        unsafe {
            use OpV::*;
            let code = frame.get_code();
            let bb = code.get_unchecked(frame.bp);
            let ins = *bb.code.get_unchecked(frame.ip);
            frame.ip += 1;
            match ins {
                Star(r) => {
                    let acc = frame.rax;
                    *frame.r(r) = acc;
                }
                Ldar(r) => {
                    let value = *frame.r(r);
                    frame.rax = value;
                }
                LdaArg(arg) => {
                    let arguments = frame.arguments;
                    if arguments.is_cell() {
                        if let CellValue::Array(ref array) = arguments.as_cell().value {
                            frame.rax = array
                                .get(arg as usize)
                                .copied()
                                .unwrap_or(Value::from(VTag::Undefined));
                        } else {
                            panic!("Arguments is not an array");
                        }
                    } else {
                        panic!("Arguments is not an array");
                    }
                }
                LdaArguments => {
                    frame.rax = frame.arguments;
                }
                Mov(r0, r1) => {
                    let value = *frame.r(r1);
                    *frame.r(r0) = value;
                }
                Add(rhs, fdbk) => {
                    let lhs = frame.rax;
                    let rhs = *frame.r(rhs);
                    if lhs.is_int32()
                        && rhs.is_int32()
                        && !((lhs.as_int32() as u32 | (rhs.as_int32() as u32 & 0xc0000000u32)) != 0)
                    {
                        // no overflow, fast path.
                        frame.rax = Value::new_int(lhs.as_int32() + rhs.as_int32());
                    } else {
                        // slow path.
                        if lhs.is_number() && rhs.is_number() {
                            frame.rax = Value::from(lhs.to_number() + rhs.to_number());
                        } else {
                            frame.rax = local_data().allocate_string(
                                format!("{}{}", lhs.to_string(), rhs.to_string()),
                                &mut frame,
                            );
                        }
                    }
                    frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize] =
                        FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
                            lhs.primitive_ty(),
                            rhs.primitive_ty(),
                            frame.rax.primitive_ty(),
                        ]));
                }
                _ => (),
            }
        }
    }
}
