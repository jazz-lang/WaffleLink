pub mod callstack;
use crate::bytecode::*;
use crate::runtime;
use cell::*;
use cgc::api::*;
use def::*;
use runtime::value::*;
use runtime::*;
use virtual_reg::*;

#[derive(Copy, Clone)]
pub enum Return {
    Error(Value),
    Return(Value),
    Yield(VirtualRegister, Value),
}

impl Runtime {
    pub extern "C" fn compare_greater(&mut self, lhs: Value, rhs: Value) -> bool {
        if lhs.is_undefined_or_null() || rhs.is_undefined_or_null() {
            return false;
        }
        if lhs.is_int32() && rhs.is_int32() {
            return lhs.as_int32() > rhs.as_int32();
        } else if lhs.is_number() && rhs.is_number() {
            return lhs.to_number() > rhs.to_number();
        } else if lhs.is_cell() && rhs.is_cell() {
            match (&lhs.as_cell().value, &rhs.as_cell().value) {
                (CellValue::String(x), CellValue::String(y)) => x > y,
                (CellValue::Array(x), CellValue::Array(y)) => x.len() > y.len(),
                (CellValue::ByteArray(x), CellValue::ByteArray(y)) => {
                    return x > y;
                }
                _ => false,
            }
        } else {
            false
        }
    }
    pub extern "C" fn compare_less(&mut self, lhs: Value, rhs: Value) -> bool {
        if lhs.is_undefined_or_null() || rhs.is_undefined_or_null() {
            return false;
        }
        if lhs.is_int32() && rhs.is_int32() {
            return lhs.as_int32() < rhs.as_int32();
        } else if lhs.is_number() && rhs.is_number() {
            return lhs.to_number() < rhs.to_number();
        } else if lhs.is_cell() && rhs.is_cell() {
            match (&lhs.as_cell().value, &rhs.as_cell().value) {
                (CellValue::String(x), CellValue::String(y)) => x < y,
                (CellValue::Array(x), CellValue::Array(y)) => x.len() < y.len(),
                (CellValue::ByteArray(x), CellValue::ByteArray(y)) => {
                    return x < y;
                }
                _ => false,
            }
        } else {
            false
        }
    }
    pub extern "C" fn compare_equal(&mut self, lhs: Value, rhs: Value) -> bool {
        if lhs.is_undefined_or_null() || rhs.is_undefined_or_null() {
            return false;
        }
        if lhs.is_int32() && rhs.is_int32() {
            return lhs.as_int32() == rhs.as_int32();
        } else if lhs.is_number() && rhs.is_number() {
            return lhs.to_number() == rhs.to_number();
        } else if lhs.is_cell() && rhs.is_cell() {
            match (&lhs.as_cell().value, &rhs.as_cell().value) {
                (CellValue::String(x), CellValue::String(y)) => x == y,
                (CellValue::Array(x), CellValue::Array(y)) => {
                    if x.len() != y.len() {
                        return false;
                    }
                    x.iter()
                        .zip(y.iter())
                        .all(|(x, y)| self.compare_equal(*x, *y))
                }
                (CellValue::ByteArray(x), CellValue::ByteArray(y)) => {
                    return x == y;
                }
                _ => lhs.as_cell().inner() == rhs.as_cell().inner(),
            }
        } else {
            lhs == rhs
        }
    }
    pub extern "C" fn compare_greater_equal(&mut self, lhs: Value, rhs: Value) -> bool {
        if lhs.is_undefined_or_null() || rhs.is_undefined_or_null() {
            return false;
        }
        if lhs.is_int32() && rhs.is_int32() {
            return lhs.as_int32() >= rhs.as_int32();
        } else if lhs.is_number() && rhs.is_number() {
            return lhs.to_number() >= rhs.to_number();
        } else if lhs.is_cell() && rhs.is_cell() {
            match (&lhs.as_cell().value, &rhs.as_cell().value) {
                (CellValue::String(x), CellValue::String(y)) => x >= y,
                (CellValue::Array(x), CellValue::Array(y)) => {
                    if x.len() > y.len() {
                        return true;
                    }
                    if x.len() != y.len() {
                        return false;
                    }
                    x.iter()
                        .zip(y.iter())
                        .all(|(x, y)| self.compare_equal(*x, *y))
                }
                (CellValue::ByteArray(x), CellValue::ByteArray(y)) => {
                    return x >= y;
                }
                _ => false,
            }
        } else {
            false
        }
    }
    pub extern "C" fn compare_less_equal(&mut self, lhs: Value, rhs: Value) -> bool {
        if lhs.is_undefined_or_null() || rhs.is_undefined_or_null() {
            return false;
        }
        if lhs.is_int32() && rhs.is_int32() {
            return lhs.as_int32() <= rhs.as_int32();
        } else if lhs.is_number() && rhs.is_number() {
            return lhs.to_number() <= rhs.to_number();
        } else if lhs.is_cell() && rhs.is_cell() {
            match (&lhs.as_cell().value, &rhs.as_cell().value) {
                (CellValue::String(x), CellValue::String(y)) => x <= y,
                (CellValue::Array(x), CellValue::Array(y)) => {
                    if x.len() < y.len() {
                        return true;
                    }
                    if x.len() != y.len() {
                        return false;
                    }
                    x.iter()
                        .zip(y.iter())
                        .all(|(x, y)| self.compare_equal(*x, *y))
                }
                (CellValue::ByteArray(x), CellValue::ByteArray(y)) => {
                    return x <= y;
                }
                _ => false,
            }
        } else {
            false
        }
    }
    pub fn interpret(&mut self) -> Return {
        let mut current = self.stack.current_frame();
        loop {
            let bp = current.bp;
            let ip = current.ip;
            let ins = current.code.code[bp].code[ip];
            current.ip += 1;
            match ins {
                Ins::Mov { dst, src } => {
                    let src = current.r(src);
                    let r = current.r_mut(dst);
                    *r = src;
                }
                Ins::Return { val } => {
                    let val = current.r(val);
                    if current.exit_on_return || self.stack.stack.len() == 1 {
                        return Return::Return(val);
                    }
                    self.stack.pop();
                    current = self.stack.current_frame();
                }
                Ins::Yield { dst, res } => {
                    return Return::Yield(dst, current.r(res));
                }
                Ins::LoadI32 { dst, imm } => {
                    *current.r_mut(dst) = Value::new_int(imm);
                }
                Ins::Safepoint => {
                    self.heap.safepoint();
                }
                Ins::LoopHint { fdbk: _ } => {
                    current.code.hotness = current.code.hotness.wrapping_add(1);
                    if current.code.hotness >= 10000 {
                        // TODO: JIT compile or OSR
                    } else {
                        continue;
                    }
                }
                Ins::Jump { dst } => {
                    current.bp = dst as _;
                    current.ip = 0;
                }
                Ins::JumpConditional {
                    cond,
                    if_true,
                    if_false,
                } => {
                    let cond = current.r(cond);
                    if cond.to_boolean() {
                        current.bp = if_true as _;
                    } else {
                        current.bp = if_false as _;
                    }
                    current.ip = 0;
                }
                Ins::LoadThis { dst } => {
                    let this = current.this;
                    *current.r_mut(dst) = this;
                }
                Ins::LoadGlobal { dst, name } => {
                    let name = current.r(name).to_string(self);
                    let name = match name {
                        Ok(x) => x,
                        Err(e) => return Return::Error(e),
                    };
                    let global = self.globals.get(&name).copied();
                    if global.is_none() {
                        return Return::Error(Value::from(
                            self.allocate_string(format!("Global '{}' not found", name)),
                        ));
                    }
                    let global = global.unwrap();
                    *current.r_mut(dst) = global;
                }
                Ins::Throw { src } => {
                    let value = current.r(src);
                    return Return::Error(value);
                }
                Ins::TryCatch { try_, catch, reg } => {
                    current.bp = try_ as _;
                    current.ip = 0;
                    current.handlers.push((catch as _, reg));
                }
                Ins::Add { dst, lhs, src, .. } => {
                    let y = current.r(src);
                    let x = current.r(lhs);
                    if x.is_int32() && y.is_int32() {
                        *current.r_mut(dst) = Value::new_int(x.as_int32() + y.as_int32());
                    } else {
                        *current.r_mut(dst) = Value::number(x.to_number() + y.to_number());
                    }
                }
                Ins::Sub { dst, lhs, src, .. } => {
                    let y = current.r(src);
                    let x = current.r(lhs);
                    if x.is_int32() && y.is_int32() {
                        *current.r_mut(dst) = Value::new_int(x.as_int32() - y.as_int32());
                    } else {
                        *current.r_mut(dst) = Value::number(x.to_number() - y.to_number());
                    }
                }
                Ins::Div { dst, lhs, src, .. } => {
                    let y = current.r(src);
                    let x = current.r(lhs);
                    if (x.is_int32() && y.is_int32()) && (y.as_int32() != 0 && x.as_int32() != 0) {
                        *current.r_mut(dst) = Value::new_int(x.as_int32() / y.as_int32());
                    } else {
                        *current.r_mut(dst) = Value::number(x.to_number() / y.to_number());
                    }
                }
                Ins::Mul { dst, lhs, src, .. } => {
                    let y = current.r(src);
                    let x = current.r(lhs);
                    if x.is_int32() && y.is_int32() {
                        *current.r_mut(dst) = Value::new_int(x.as_int32() * y.as_int32());
                    } else {
                        *current.r_mut(dst) = Value::number(x.to_number() * y.to_number());
                    }
                }
                Ins::Mod { dst, lhs, src, .. } => {
                    let y = current.r(src);
                    let x = current.r(lhs);
                    *current.r_mut(dst) = Value::number(x.to_number() % y.to_number());
                }
                Ins::Eq { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs);
                    let rhs = current.r(src);
                    let result = self.compare_equal(lhs, rhs);
                    *current.r_mut(dst) = if result {
                        Value::true_()
                    } else {
                        Value::false_()
                    };
                }
                Ins::NEq { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs);
                    let rhs = current.r(src);
                    let result = !self.compare_equal(lhs, rhs);
                    *current.r_mut(dst) = if result {
                        Value::true_()
                    } else {
                        Value::false_()
                    };
                }
                Ins::Greater { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs);
                    let rhs = current.r(src);
                    let result = self.compare_greater(lhs, rhs);
                    *current.r_mut(dst) = if result {
                        Value::true_()
                    } else {
                        Value::false_()
                    };
                }
                Ins::Less { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs);
                    let rhs = current.r(src);
                    let result = self.compare_less(lhs, rhs);
                    *current.r_mut(dst) = if result {
                        Value::true_()
                    } else {
                        Value::false_()
                    };
                }
                Ins::LessEq { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs);
                    let rhs = current.r(src);
                    let result = self.compare_less_equal(lhs, rhs);
                    *current.r_mut(dst) = if result {
                        Value::true_()
                    } else {
                        Value::false_()
                    };
                }
                Ins::GreaterEq { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs);
                    let rhs = current.r(src);
                    let result = self.compare_greater_equal(lhs, rhs);
                    *current.r_mut(dst) = if result {
                        Value::true_()
                    } else {
                        Value::false_()
                    };
                }
                Ins::Shr { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs).to_int32();
                    let rhs = current.r(src).to_int32();
                    *current.r_mut(dst) = Value::new_int(lhs >> (rhs & 0x1f));
                }
                Ins::UShr { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs).to_uint32();
                    let rhs = current.r(src).to_uint32();
                    *current.r_mut(dst) = Value::new_int((lhs << rhs & 0x1f) as i32);
                }
                Ins::Shl { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs).to_int32();
                    let rhs = current.r(src).to_int32();
                    *current.r_mut(dst) = Value::new_int(lhs << (rhs & 0x1f));
                }
                Ins::Concat { dst, lhs, src } => {
                    let lhs = current.r(lhs);
                    let rhs = current.r(src);
                    let s1 = lhs.to_string(self);
                    let s2 = rhs.to_string(self);
                    let s1 = match s1 {
                        Ok(v) => v,
                        Err(e) => return Return::Error(e),
                    };
                    let s2 = match s2 {
                        Ok(v) => v,
                        Err(e) => return Return::Error(e),
                    };
                    let s = format!("{}{}", s1, s2);
                    *current.r_mut(dst) = Value::from(self.allocate_string(s));
                }
                Ins::LoadUp { dst, up } => {
                    let func = current.func;
                    if let CellValue::Array(ref arr) = func.as_cell().value {
                        *current.r_mut(dst) = arr[up as usize];
                    } else {
                        panic!("Function does not have environment");
                    }
                }
                Ins::Call {
                    dst,
                    function,
                    this,
                    begin,
                    end,
                } => {
                    assert!(begin.is_argument());
                    assert!(end.is_argument());
                    let arguments = {
                        let mut v = vec![];
                        for x in begin.to_argument()..=end.to_argument() {
                            v.push(current.r(VirtualRegister::argument(x)));
                        }
                        v
                    };
                    let function = current.r(function);
                    let this = current.r(this);
                    match self.call(function, this, &arguments) {
                        Ok(val) => {
                            *current.r_mut(dst) = val;
                        }
                        Err(e) => return Return::Error(e),
                    }
                }
                Ins::GetById {
                    dst,
                    base,
                    id,
                    fdbk: _,
                } => {
                    let base = current.r(base);
                    let id = current.r(id);
                    *current.r_mut(dst) = match base.lookup(self, id) {
                        Ok(val) => val.unwrap_or(Value::undefined()),
                        Err(e) => return Return::Error(e),
                    }
                }
                Ins::GetByVal { dst, base, val } => {
                    let base = current.r(base);
                    let id = current.r(val);
                    *current.r_mut(dst) = match base.lookup(self, id) {
                        Ok(val) => val.unwrap_or(Value::undefined()),
                        Err(e) => return Return::Error(e),
                    }
                }
                Ins::PutById { val, base, id } => {
                    let base = current.r(base);
                    let val = current.r(val);
                    let key = current.r(id);
                    match base.put(self, key, val) {
                        Err(e) => return Return::Error(e),
                        _ => (),
                    }
                }
                Ins::PutByVal { src, base, val } => {
                    let base = current.r(base);
                    let key = current.r(val);
                    let val = current.r(src);
                    match base.put(self, key, val) {
                        Err(e) => return Return::Error(e),
                        _ => (),
                    }
                }
                /*Ins::Await { dst, on } => {
                    let maybe_future = current.r(on);
                    if maybe_future.is_cell() {
                        let val = maybe_future.as_cell().take_value();
                        if let CellValue::Future(fut) = val {
                            let res = fut.await;
                            match res {
                                Return::Error(err) => return Return::Error(err),
                                Return::Return(val) => {
                                    *current.r_mut(dst) = val;
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            maybe_future.as_cell().value = val;
                            return Return::Error(Value::from(
                                self.allocate_string("Future expected"),
                            ));
                        }
                    } else {
                        return Return::Error(Value::from(self.allocate_string("Future expected")));
                    }
                }*/
                _ => unimplemented!("TODO!"),
            }
        }
    }
}
