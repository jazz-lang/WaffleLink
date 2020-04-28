use crate::arc::ArcWithoutWeak as Arc;
use crate::bytecode::op::*;
use crate::heap::*;
use crate::runtime;
use runtime::cell::*;
use runtime::frame::*;
use runtime::function::*;
use runtime::process::*;
use runtime::symbol::*;
use runtime::value::*;
use OpV::*;
pub fn run(mut frame: Frame) -> Result<Value, Value> {
    'interp: loop {
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
                        && !(lhs.as_int32() > std::i32::MAX - rhs.as_int32())
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
                Sub(rhs, fdbk) => {
                    let lhs = frame.rax;
                    let rhs = *frame.r(rhs);
                    match () {
                        _ if lhs.is_int32() && rhs.is_int32() => {
                            match lhs.as_int32().overflowing_sub(rhs.as_int32()) {
                                (x, false) => {
                                    frame.rax = Value::new_int(x);
                                }
                                _ => (),
                            }
                        }
                        _ => {
                            if lhs.is_number() && rhs.is_number() {
                                frame.rax = Value::from(lhs.to_number() + rhs.to_number());
                            } else {
                                frame.rax = Value::from(std::f64::NAN);
                            }
                        }
                    }
                    frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize] =
                        FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
                            lhs.primitive_ty(),
                            rhs.primitive_ty(),
                            frame.rax.primitive_ty(),
                        ]));
                }
                Mul(rhs, fdbk) => {
                    let lhs = frame.rax;
                    let rhs = *frame.r(rhs);
                    match () {
                        _ if lhs.is_int32() && rhs.is_int32() => {
                            match lhs.as_int32().overflowing_mul(rhs.as_int32()) {
                                (x, false) => {
                                    frame.rax = Value::new_int(x);
                                }
                                _ => (),
                            }
                        }
                        _ => {
                            if lhs.is_number() && rhs.is_number() {
                                frame.rax = Value::from(lhs.to_number() + rhs.to_number());
                            } else {
                                frame.rax = Value::from(std::f64::NAN);
                            }
                        }
                    }
                    frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize] =
                        FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
                            lhs.primitive_ty(),
                            rhs.primitive_ty(),
                            frame.rax.primitive_ty(),
                        ]));
                }
                Div(rhs, fdbk) => {
                    let lhs = frame.rax;
                    let rhs = *frame.r(rhs);
                    frame.rax = Value::from(lhs.to_number() / rhs.to_number());
                    frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize] =
                        FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
                            lhs.primitive_ty(),
                            rhs.primitive_ty(),
                            frame.rax.primitive_ty(),
                        ]));
                }
                Mod(rhs, fdbk) => {
                    let lhs = frame.rax;
                    let rhs = *frame.r(rhs);
                    if lhs.is_int32() && rhs.is_int32() && rhs.as_int32() != 0 {
                        frame.rax = Value::new_int(lhs.as_int32() % rhs.as_int32());
                    } else {
                        frame.rax = Value::from(lhs.to_number() % rhs.to_number());
                    }
                    frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize] =
                        FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
                            lhs.primitive_ty(),
                            rhs.primitive_ty(),
                            frame.rax.primitive_ty(),
                        ]));
                }
                Shl(rhs, fdbk) => {
                    let val = frame.rax;
                    let shift = *frame.r(rhs);
                    if val.is_int32() && shift.is_int32() {
                        frame.rax = Value::new_int(val.as_int32() << (shift.as_int32() & 0x1f));
                    } else {
                        let val = val.to_number().floor() as i32;
                        let shift = shift.to_number().floor() as i32;
                        frame.rax = Value::new_int(val << (shift & 0x1f));
                    }
                    frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize] =
                        FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
                            val.primitive_ty(),
                            shift.primitive_ty(),
                            frame.rax.primitive_ty(),
                        ]));
                }
                Shr(rhs, fdbk) => {
                    let val = frame.rax;
                    let shift = *frame.r(rhs);
                    if val.is_int32() && shift.is_int32() {
                        frame.rax = Value::new_int(val.as_int32() >> (shift.as_int32() & 0x1f));
                    } else {
                        let val = val.to_int32();
                        let shift = shift.to_int32();
                        frame.rax = Value::new_int(val >> (shift & 0x1f));
                    }
                    frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize] =
                        FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
                            val.primitive_ty(),
                            shift.primitive_ty(),
                            frame.rax.primitive_ty(),
                        ]));
                }
                UShr(rhs, fdbk) => {
                    let val = frame.rax;
                    let shift = *frame.r(rhs);
                    if val.is_int32() && shift.is_int32() {
                        frame.rax = Value::new_int(val.as_int32() >> (shift.as_int32() & 0x1f));
                    } else {
                        let val = val.to_int32();
                        let shift = shift.to_int32();
                        frame.rax = Value::new_int(((val as u32) >> (shift as u32 & 0x1f)) as i32);
                    }
                    frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize] =
                        FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
                            val.primitive_ty(),
                            shift.primitive_ty(),
                            frame.rax.primitive_ty(),
                        ]));
                }
                BitwiseOr(src2, fdbk) => {
                    let src1 = frame.rax;
                    let src2 = *frame.r(src2);
                    if src1.is_int32() && src2.is_int32() {
                        frame.rax = Value::new_int(src1.as_int32() | src2.as_int32());
                    } else {
                        frame.rax = Value::new_int(src1.to_int32() | src2.to_int32() as i32);
                    }
                    frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize] =
                        FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
                            src1.primitive_ty(),
                            src2.primitive_ty(),
                            frame.rax.primitive_ty(),
                        ]));
                }
                BitwiseAnd(src2, fdbk) => {
                    let src1 = frame.rax;
                    let src2 = *frame.r(src2);
                    if src1.is_int32() && src2.is_int32() {
                        frame.rax = Value::new_int(src1.as_int32() & src2.as_int32());
                    } else {
                        frame.rax = Value::new_int(src1.to_int32() & src2.to_int32() as i32);
                    }
                    frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize] =
                        FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
                            src1.primitive_ty(),
                            src2.primitive_ty(),
                            frame.rax.primitive_ty(),
                        ]));
                }
                BitwiseXor(src2, fdbk) => {
                    let src1 = frame.rax;
                    let src2 = *frame.r(src2);
                    if src1.is_int32() && src2.is_int32() {
                        frame.rax = Value::new_int(src1.as_int32() ^ src2.as_int32());
                    } else {
                        frame.rax = Value::new_int(src1.to_int32() ^ src2.to_int32() as i32);
                    }
                    frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize] =
                        FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
                            src1.primitive_ty(),
                            src2.primitive_ty(),
                            frame.rax.primitive_ty(),
                        ]));
                }
                LdaUndefined => {
                    frame.rax = Value::from(VTag::Undefined);
                }
                LdaInt(x) => {
                    frame.rax = Value::new_int(x);
                }
                LdaNull => {
                    frame.rax = Value::from(VTag::Null);
                }
                LdaGlobal(key) => {
                    let key = frame.get_constant(key);
                    let global = local_data().globals.get(&Symbol::new_value(key));
                    frame.rax = global.copied().unwrap_or(Value::from(VTag::Undefined));
                }
                StaGlobal(key) => {
                    let key = frame.get_constant(key);
                    local_data()
                        .globals
                        .insert(Symbol::new_value(key), frame.rax);
                }
                LdaGlobalDirect(_) => unimplemented!(),
                StaGlobalDirect(_) => unimplemented!(),
                LdaById(base_r, key_r, fdbk) => {
                    /*let bp = frame.bp;
                    let ip = frame.ip;
                    let feedback =
                        &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];
                    let mut should_cache = true;
                    let mut misses = 0;
                    if let FeedBack::Cache(_, _, m) = feedback {
                        if *m >= 15 {
                            should_cache = false;
                        }
                        misses = *m;
                    }
                    let mut base = *frame.r(base_r);
                    let key = Symbol::new_value(frame.get_constant(key_r));
                    let mut slot = Slot::new();
                    if base.is_cell() && should_cache {
                        let mut cell = base.as_cell();
                        if cell.lookup(key, &mut slot) {
                            if slot.base.raw == cell.raw {
                                frame.get_code_mut()[bp].code[ip - 1] =
                                    LdaOwnProperty(base_r, key_r, fdbk);
                            } else {
                                if let Some(proto) = cell.prototype {
                                    if slot.base.raw == proto.raw {
                                        frame.get_code_mut()[bp].code[ip - 1] =
                                            LdaProtoProperty(base_r, key_r, fdbk);
                                    } else {
                                        frame.get_code_mut()[bp].code[ip - 1] =
                                            LdaChainProperty(base_r, key_r, fdbk);
                                    }
                                } else {
                                    unreachable!()
                                }
                            }
                            frame.rax = slot.value();
                            let feedback =
                                &mut frame.func.func_value_unchecked_mut().feedback_vector
                                    [fdbk as usize];
                            *feedback = FeedBack::Cache(slot.base.attributes, slot.offset, misses);
                        } else {
                            frame.rax = slot.value();
                        }
                    } else {
                        base.lookup(key, &mut slot);
                        frame.rax = slot.value();
                        if !should_cache {
                            frame.get_code_mut()[bp].code[ip - 1] =
                                OpV::LdaSlowById(base_r, key_r, fdbk);
                        }
                    }*/
                    lda_by_id(&mut frame, base_r, key_r, fdbk);
                }
                StaById(base_r, key_r, fdbk) => {
                    /*let bp = frame.bp;
                    let ip = frame.ip;
                    let feedback =
                        &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];
                    let mut should_cache = true;
                    let mut misses = 0;
                    if let FeedBack::Cache(_, _, m) = feedback {
                        if *m >= 15 {
                            should_cache = false;
                        }
                        misses = *m;
                    }
                    let mut base = *frame.r(base_r);
                    let key = Symbol::new_value(frame.get_constant(key_r));
                    let mut slot = Slot::new();
                    if base.is_cell() && should_cache {
                        let mut cell = base.as_cell();
                        cell.insert(key, frame.rax, &mut slot);
                        frame.get_code_mut()[bp].code[ip - 1] = StaOwnProperty(base_r, key_r, fdbk);
                        let feedback = &mut frame.func.func_value_unchecked_mut().feedback_vector
                            [fdbk as usize];

                        *feedback = FeedBack::Cache(cell.attributes, slot.offset, misses);
                    } else {
                        base.insert(key, frame.rax, &mut slot);
                        if !should_cache {
                            frame.get_code_mut()[bp].code[ip - 1] =
                                StaSlowById(base_r, key_r, fdbk);
                        }
                    }*/
                    sta_by_id(&mut frame, base_r, key_r, fdbk)
                }
                LdaByVal(base, val) => {
                    let mut base = *frame.r(base);
                    let val = *frame.r(val);
                    let mut slot = Slot::new();
                    base.lookup(Symbol::new_value(val), &mut slot);
                    frame.rax = slot.value();
                }
                StaByVal(base, val) => {
                    let mut base = *frame.r(base);
                    let val = *frame.r(val);
                    let mut slot = Slot::new();
                    base.insert(Symbol::new_value(val), &mut slot);
                    slot.store(frame.rax);
                    frame.rax = slot.value();
                }
                LdaByIdx(base_r, idx_r, fdbk) => {
                    lda_by_idx(&mut frame, base_r, idx_r, fdbk);
                }
                StaByIdx(base_r, key_r, fdbk) => sta_by_idx(&mut frame, base_r, key_r, fdbk),
                LdaOwnProperty(base_r, key_r, fdbk) => {
                    let mut base = *frame.r(base_r);
                    let key = frame.get_constant(key_r);
                    let mut slot = Slot::new();
                    let feedback =
                        &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];

                    if let FeedBack::Cache(attrs, offset, misses) = feedback {
                        if base.is_cell() {
                            if base.as_cell().attributes.ptr_eq(attrs) {
                                frame.rax = base.as_cell().direct(*offset);
                            } else {
                                *misses += 1;
                                lda_by_id(&mut frame, base_r, key_r, fdbk);
                            }
                        } else {
                            base.lookup(Symbol::new_value(key), &mut slot);
                            frame.rax = slot.value();
                        }
                    } else {
                        unreachable!();
                    };
                }
                StaOwnProperty(base_r, key_r, fdbk) => {
                    let base = *frame.r(base_r);
                    let value = frame.rax;

                    let key = frame.get_constant(key_r);
                    let mut slot = Slot::new();
                    let feedback =
                        &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];

                    if let FeedBack::Cache(attrs, offset, misses) = feedback {
                        if base.is_cell() {
                            if base.as_cell().attributes.ptr_eq(attrs) {
                                base.as_cell().store_direct(*offset, value);
                            } else {
                                *misses += 1;
                                sta_by_id(&mut frame, base_r, key_r, fdbk);
                            }
                        } else {
                            base.as_cell().insert(Symbol::new_value(key), &mut slot);
                            slot.store(value);
                        }
                    } else {
                        unreachable!();
                    }
                }
                LdaProtoProperty(base_r, key_r, fdbk) => {
                    let mut base = *frame.r(base_r);

                    let key = frame.get_constant(key_r);
                    let mut slot = Slot::new();
                    let feedback =
                        &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];

                    if let FeedBack::Cache(attrs, offset, misses) = feedback {
                        if base.is_cell() {
                            if let Some(proto) = base.as_cell().prototype {
                                if proto.attributes.ptr_eq(attrs) {
                                    frame.rax = proto.direct(*offset);
                                } else {
                                    *misses += 1;
                                    lda_by_id(&mut frame, base_r, key_r, fdbk);
                                }
                            } else {
                                *misses += 1;
                                lda_by_id(&mut frame, base_r, key_r, fdbk);
                            }
                        } else {
                            base.lookup(Symbol::new_value(key), &mut slot);
                            frame.rax = slot.value();
                        }
                    }
                }
                LdaChainProperty(base_r, key_r, fdbk) => {
                    let mut base = *frame.r(base_r);

                    let key = frame.get_constant(key_r);
                    let mut slot = Slot::new();
                    let feedback =
                        &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];

                    if let FeedBack::Cache(attrs, offset, misses) = feedback {
                        if base.is_cell() {
                            let mut obj = Some(base.as_cell());
                            while let Some(object) = obj {
                                if object.attributes.ptr_eq(attrs) {
                                    frame.rax = object.direct(*offset);
                                    continue 'interp;
                                } else {
                                    obj = object.prototype;
                                }
                            }
                            *misses += 1;
                            lda_by_id(&mut frame, base_r, key_r, fdbk);
                        } else {
                            base.lookup(Symbol::new_value(key), &mut slot);
                            frame.rax = slot.value();
                        }
                    }
                }
                LdaOwnIdx(base_r, idx_r, fdbk) => {
                    let mut base = *frame.r(base_r);

                    let key = Symbol::new_index(idx_r as _);
                    let mut slot = Slot::new();
                    let feedback =
                        &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];

                    if let FeedBack::Cache(attrs, offset, misses) = feedback {
                        if base.is_cell() {
                            if base.as_cell().attributes.ptr_eq(attrs) {
                                frame.rax = base.as_cell().direct(*offset);
                            } else {
                                *misses += 1;
                                lda_by_idx(&mut frame, base_r, idx_r, fdbk);
                            }
                        } else {
                            base.lookup(key, &mut slot);
                            frame.rax = slot.value();
                        }
                    }
                }
                LdaProtoIdx(base_r, idx_r, fdbk) => {
                    let mut base = *frame.r(base_r);

                    let key = Symbol::new_index(idx_r as _);
                    let mut slot = Slot::new();
                    let feedback =
                        &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];

                    if let FeedBack::Cache(attrs, offset, misses) = feedback {
                        if base.is_cell() {
                            if base.as_cell().attributes.ptr_eq(attrs) {
                                frame.rax = base.as_cell().direct(*offset);
                            } else {
                                *misses += 1;
                                lda_by_idx(&mut frame, base_r, idx_r, fdbk);
                            }
                        } else {
                            base.lookup(key, &mut slot);
                            frame.rax = slot.value();
                        }
                    }
                }
                LdaChainIdx { .. } => unimplemented!(),
                StaOwnIdx(base_r, idx_r, fdbk) => {
                    let mut base = *frame.r(base_r);

                    let key = Symbol::new_index(idx_r as _);
                    let mut slot = Slot::new();
                    let feedback =
                        &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];

                    if let FeedBack::Cache(attrs, offset, misses) = feedback {
                        if base.is_cell() {
                            if base.as_cell().attributes.ptr_eq(attrs) {
                                base.as_cell().store_direct(*offset, frame.rax);
                            } else {
                                *misses += 1;
                                sta_by_idx(&mut frame, base_r, idx_r, fdbk);
                            }
                        } else {
                            /*base.lookup(key, &mut slot);
                            frame.rax = slot.value();*/
                            base.insert(key, &mut slot);
                            slot.store(frame.rax);
                        }
                    }
                }
                LdaSlowById(base_r, key_r, _) => {
                    let mut base = *frame.r(base_r);
                    let key = Symbol::new_value(frame.get_constant(key_r));
                    let mut slot = Slot::new();
                    base.lookup(key, &mut slot);
                    frame.rax = slot.value();
                }
                StaSlowById(base_r, key_r, _) => {
                    let mut base = *frame.r(base_r);
                    let key = Symbol::new_value(frame.get_constant(key_r));
                    let mut slot = Slot::new();
                    base.insert(key, &mut slot);
                    if !slot.value.is_null() {
                        *slot.value = frame.rax;
                    }
                }
                LdaSlowByIdx(base_r, key, _) => {
                    let mut base = *frame.r(base_r);
                    let key = Symbol::new_index(key as _);
                    let mut slot = Slot::new();
                    base.lookup(key, &mut slot);
                    frame.rax = slot.value();
                }
                StaSlowByIdx(base_r, key, _) => {
                    let mut base = *frame.r(base_r);
                    let key = Symbol::new_index(key as _);
                    let mut slot = Slot::new();
                    base.insert(key, &mut slot);
                    slot.store(frame.rax);
                }
                PushA => {
                    let val = frame.rax;
                    frame.push(val);
                }
                PopA => {
                    let val = frame.pop().unwrap_or(Value::from(VTag::Undefined));
                    frame.rax = val;
                }
                PushR(r) => {
                    let val = *frame.r(r);
                    frame.push(val);
                }
                PopR(r) => {
                    let val = frame.pop().unwrap_or(Value::from(VTag::Undefined));
                    *frame.r(r) = val;
                }
                LdaThis => {
                    let this = frame.this;
                    frame.rax = this;
                }
                Call(function, argc) => {
                    let value = *frame.r(function);
                    let this = frame.rax;
                    let arguments = {
                        let mut v = vec![];
                        for _ in 0..argc {
                            if let Some(val) = frame.pop() {
                                v.push(val);
                            } else {
                                break;
                            }
                        }
                        v
                    };
                    if value.is_cell() {
                        let cell = value.as_cell();
                        if cell.is_function() {
                            let func = cell.func_value_unchecked();
                            let args = local_data().allocate_array(arguments, &mut frame);
                            /*let mut new_frame = Frame::new(frame.rax, func.module);
                            new_frame.arguments =

                            new_frame.code*/
                            match func.code {
                                FunctionCode::Bytecode(_) => {
                                    match func.threshold {
                                        x if x >= 10_000 => {
                                            // TODO: Simple JIT
                                        }
                                        x if x >= 100_000 => {
                                            // TODO: Full JIT
                                        }
                                        _ => {
                                            local_data().frames.push(frame);
                                            frame = Frame::new(this, func.module);
                                            frame.func = cell;
                                            frame.arguments = args;
                                            continue;
                                        }
                                    }
                                }
                                FunctionCode::Native(fun) => {
                                    let mut f = Frame::native_frame(frame.rax, args, func.module);
                                    let value = fun(&mut f)?;
                                    frame.rax = value;
                                }
                            }
                        } else {
                            return Err(local_data().allocate_string(
                                format!("{} is not a function", value.to_string()),
                                &mut frame,
                            ));
                        }
                    } else {
                        return Err(local_data().allocate_string(
                            format!("{} is not a function", value.to_string()),
                            &mut frame,
                        ));
                    }
                }
                Throw => {
                    let err = frame.rax;
                    local_data().frames.push(frame);
                    return Err(err);
                }
                CatchSetup(block) => {
                    frame.try_catch.push(block as _);
                }
                BrC(if_true, if_false) => {
                    let c = frame.rax.to_boolean();
                    if c {
                        frame.bp = if_true as _;
                        frame.ip = 0;
                    } else {
                        frame.bp = if_false as _;
                        frame.ip = 0;
                    }
                }
                Br(x) => {
                    frame.bp = x as _;
                    frame.ip = 0;
                }
                Return => return Ok(frame.rax),
                _ => (),
            }
        }
    }

    fn sta_by_idx(frame: &mut Frame, base_r: u8, key_r: u32, fdbk: u32) {
        let bp = frame.bp;
        let ip = frame.ip;
        let feedback = &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];
        let mut should_cache = true;
        let mut misses = 0;
        if let FeedBack::Cache(_, _, m) = feedback {
            if *m >= 15 {
                should_cache = false;
            }
            misses = *m;
        }
        let mut base = *frame.r(base_r);
        let key = Symbol::new_index(key_r as i32);
        let mut slot = Slot::new();
        if base.is_cell() && should_cache {
            let mut cell = base.as_cell();
            cell.insert(key, &mut slot);
            if !slot.value.is_null() {
                *slot.value = frame.rax;
            }
            frame.get_code_mut()[bp].code[ip - 1] = StaOwnIdx(base_r, key_r, fdbk);
            let feedback =
                &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];
            *feedback = FeedBack::Cache(cell.attributes.clone(), slot.offset, misses);
        } else {
            base.insert(key, &mut slot);
            slot.store(frame.rax);
            if !should_cache {
                frame.get_code_mut()[bp].code[ip - 1] = StaSlowByIdx(base_r, key_r, fdbk);
            }
        }
    }
    fn lda_by_idx(frame: &mut Frame, base_r: u8, idx_r: u32, fdbk: u32) {
        let bp = frame.bp;
        let ip = frame.ip;
        let feedback = &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];
        let mut should_cache = true;
        let mut misses = 0;
        if let FeedBack::Cache(_, _, m) = feedback {
            if *m >= 15 {
                should_cache = false;
            }
            misses = *m;
        }
        let mut base = *frame.r(base_r);
        let key = Symbol::new_index(idx_r as i32);
        let mut slot = Slot::new();
        if base.is_cell() && should_cache {
            let mut cell = base.as_cell();
            if cell.lookup(key, &mut slot) {
                if slot.base.raw == cell.raw {
                    frame.get_code_mut()[bp].code[ip - 1] = LdaOwnIdx(base_r, idx_r, fdbk);
                } else {
                    if let Some(proto) = cell.prototype {
                        if slot.base.raw == proto.raw {
                            frame.get_code_mut()[bp].code[ip - 1] =
                                LdaProtoIdx(base_r, idx_r, fdbk);
                        } else {
                            frame.get_code_mut()[bp].code[ip - 1] =
                                LdaChainIdx(base_r, idx_r, fdbk);
                        }
                    } else {
                        unreachable!()
                    }
                }
                frame.rax = slot.value();
                let feedback =
                    &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];
                *feedback = FeedBack::Cache(slot.base.attributes.clone(), slot.offset, misses);
            } else {
                frame.rax = slot.value();
            }
        } else {
            base.lookup(key, &mut slot);
            frame.rax = slot.value();
            if !should_cache {
                frame.get_code_mut()[bp].code[ip - 1] = OpV::LdaSlowByIdx(base_r, idx_r, fdbk);
            }
        }
    }

    fn sta_by_id(frame: &mut Frame, base_r: u8, key_r: u16, fdbk: u32) {
        let bp = frame.bp;
        let ip = frame.ip;
        let feedback = &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];
        let mut should_cache = true;
        let mut misses = 0;
        if let FeedBack::Cache(_, _, m) = feedback {
            if *m >= 15 {
                should_cache = false;
            }
            misses = *m;
        }
        let mut base = *frame.r(base_r);
        let key = Symbol::new_value(frame.get_constant(key_r));
        let mut slot = Slot::new();
        if base.is_cell() && should_cache {
            let mut cell = base.as_cell();
            cell.insert(key, &mut slot);
            if !slot.value.is_null() {
                *slot.value = frame.rax;
            }
            frame.get_code_mut()[bp].code[ip - 1] = StaOwnProperty(base_r, key_r, fdbk);
            let feedback =
                &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];

            *feedback = FeedBack::Cache(cell.attributes.clone(), slot.offset, misses);
        } else {
            base.insert(key, &mut slot);
            slot.store(frame.rax);
            if !should_cache {
                frame.get_code_mut()[bp].code[ip - 1] = StaSlowById(base_r, key_r, fdbk);
            }
        }
    }
    fn lda_by_id(frame: &mut Frame, base_r: u8, key_r: u16, fdbk: u32) {
        let bp = frame.bp;
        let ip = frame.ip;
        let feedback = &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];
        let mut should_cache = true;
        let mut misses = 0;
        if let FeedBack::Cache(_, _, m) = feedback {
            if *m >= 15 {
                should_cache = false;
            }
            misses = *m;
        }
        let mut base = *frame.r(base_r);
        let key = Symbol::new_value(frame.get_constant(key_r));
        let mut slot = Slot::new();
        if base.is_cell() && should_cache {
            let mut cell = base.as_cell();
            if cell.lookup(key, &mut slot) {
                if slot.base.raw == cell.raw {
                    frame.get_code_mut()[bp].code[ip - 1] = LdaOwnProperty(base_r, key_r, fdbk);
                } else {
                    if let Some(proto) = cell.prototype {
                        if slot.base.raw == proto.raw {
                            frame.get_code_mut()[bp].code[ip - 1] =
                                LdaProtoProperty(base_r, key_r, fdbk);
                        } else {
                            frame.get_code_mut()[bp].code[ip - 1] =
                                LdaChainProperty(base_r, key_r, fdbk);
                        }
                    } else {
                        unreachable!()
                    }
                }
                frame.rax = slot.value();
                let feedback =
                    &mut frame.func.func_value_unchecked_mut().feedback_vector[fdbk as usize];
                *feedback = FeedBack::Cache(slot.base.attributes.clone(), slot.offset, misses);
            } else {
                frame.rax = slot.value();
            }
        } else {
            base.lookup(key, &mut slot);
            frame.rax = slot.value();
            if !should_cache {
                frame.get_code_mut()[bp].code[ip - 1] = OpV::LdaSlowById(base_r, key_r, fdbk);
            }
        }
    }
}
