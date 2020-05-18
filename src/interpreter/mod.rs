pub mod callstack;
use crate::bytecode::*;
use crate::fullcodegen::FullCodegen;
use crate::fullcodegen::*;
use crate::heap::api::*;
use crate::interpreter::callstack::CallFrame;
use crate::jit::func::Handler;
use crate::jit::types::*;
use crate::jit::JITResult;
use crate::runtime;
use cell::*;
use def::*;
use runtime::value::*;
use runtime::*;
use virtual_reg::*;

#[derive(Copy, Clone)]
pub enum Return {
    Error(Value),
    Return(Value),
}

enum C {
    Ok(Value),
    Err(Value),
    Continue,
}

impl Runtime {
    pub extern "C" fn compare_greater(lhs: Value, rhs: Value) -> bool {
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
    pub extern "C" fn compare_less(lhs: Value, rhs: Value) -> bool {
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
    pub extern "C" fn compare_equal(lhs: Value, rhs: Value) -> bool {
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
                        .all(|(x, y)| Self::compare_equal(*x, *y))
                }
                (CellValue::ByteArray(x), CellValue::ByteArray(y)) => {
                    return x == y;
                }
                _ => lhs.as_cell().raw == rhs.as_cell().raw,
            }
        } else {
            lhs == rhs
        }
    }
    pub extern "C" fn compare_greater_equal(lhs: Value, rhs: Value) -> bool {
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
                        .all(|(x, y)| Self::compare_equal(*x, *y))
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
    pub extern "C" fn compare_less_equal(lhs: Value, rhs: Value) -> bool {
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
                        .all(|(x, y)| Self::compare_equal(*x, *y))
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
    #[inline(never)]
    extern "C" fn call_interp(&mut self, func: Value, this: Value, args: &[Value]) -> C {
        let ptr = self as *mut Self;
        if func.is_cell() == false {
            return C::Err(Value::from(self.allocate_string("not a function")));
        }
        let val = func;
        use crate::jit::*;
        use osr::*;
        if let CellValue::Function(ref mut func) = func.as_cell().value {
            match func {
                Function::Native { name: _, native } => match native(self, this, args) {
                    Return::Error(e) => return C::Err(e),
                    Return::Return(x) => return C::Ok(x),
                    _ => unimplemented!("TODO: Generators"),
                },
                Function::Regular(ref mut regular) => {
                    let regular: &mut RegularFunction = regular;
                    match regular.kind {
                        RegularFunctionKind::Generator => {
                            unimplemented!("TODO: Instantiat generator");
                        }
                        _ => {
                            if self.configs.enable_jit {
                                if let Some(ref jit) = regular.code.jit_code {
                                    let _ = self.stack.push(
                                        unsafe { &mut *ptr },
                                        val,
                                        regular.code.clone(),
                                        this,
                                    );

                                    let mut cur = self.stack.current_frame();
                                    let func: extern "C" fn(
                                        &mut Runtime,
                                        &mut CallFrame,
                                        usize,
                                    )
                                        -> JITResult =
                                        unsafe { std::mem::transmute(jit.instruction_start()) };
                                    for (i, arg) in args.iter().enumerate() {
                                        if i >= self.stack.current_frame().entries.len() {
                                            break;
                                        }
                                        self.stack.current_frame().entries[i] = *arg;
                                    }
                                    call!(before);
                                    let res = func(self, cur.get_mut(), jit.osr_table.labels[0]);
                                    call!(after);
                                    match res {
                                        JITResult::Err(e) => return C::Err(e),
                                        JITResult::Ok(x) => return C::Ok(x),
                                        JITResult::OSRExit => {
                                            /*self.stack.current_frame().ip = entry.to_ip;
                                            self.stack.current_frame().bp = entry.to_bp;
                                            match self.interpret() {
                                                Return::Return(val) => return Ok(val),
                                                Return::Error(e) => return Err(e),
                                                Return::Yield { .. } => {
                                                    unimplemented!("TODO: Generators")
                                                }
                                            }*/
                                            unimplemented!();
                                        }
                                    }
                                } else {
                                    if regular.code.hotness >= 1000 {
                                        if let RegularFunctionKind::Ordinal = regular.kind {
                                            log::trace!(
                                                "Compiling function after {} calls",
                                                regular.code.hotness / 50
                                            );
                                            let mut gen = FullCodegen::new(regular.code.clone());
                                            gen.compile(false);
                                            log::trace!(
                                                "Disassembly for '{}'",
                                                unwrap!(regular.name.to_string(self))
                                            );
                                            let code = gen.finish(self, true);
                                            let func: extern "C" fn(
                                                &mut Runtime,
                                                &mut CallFrame,
                                                usize,
                                            )
                                                -> JITResult = unsafe {
                                                std::mem::transmute(code.instruction_start())
                                            };
                                            let x = unsafe { &mut *ptr };
                                            let _ =
                                                self.stack.push(x, val, regular.code.clone(), this);
                                            let mut cur = self.stack.current_frame();
                                            for (i, arg) in args.iter().enumerate() {
                                                if i >= self.stack.current_frame().entries.len() {
                                                    break;
                                                }
                                                self.stack.current_frame().entries[i] = *arg;
                                            }
                                            let enter = code.osr_table.labels[0];
                                            regular.code.jit_code = Some(code);
                                            call!(before);
                                            let res = func(self, cur.get_mut(), enter);
                                            call!(after);
                                            //self.stack.pop();
                                            match res {
                                                JITResult::Ok(val) => {
                                                    return C::Ok(val);
                                                }
                                                JITResult::Err(e) => {
                                                    return C::Err(e);
                                                }
                                                _ => unimplemented!(),
                                            }
                                        }
                                    } else {
                                        regular.code.get_mut().hotness =
                                            regular.code.hotness.wrapping_add(50);
                                    }
                                }
                            }
                            // unsafe code block is actually safe,we just access heap.
                            match self.stack.push(
                                unsafe { &mut *ptr },
                                val,
                                regular.code.clone(),
                                this,
                            ) {
                                Err(e) => return C::Err(e),
                                _ => (),
                            }
                            for (i, arg) in args.iter().enumerate() {
                                if i >= self.stack.current_frame().entries.len() {
                                    break;
                                }
                                self.stack.current_frame().entries[i] = *arg;
                            }
                            self.stack.current_frame().exit_on_return = false;
                            /* match self.interpret() {
                                Return::Return(val) => return Ok(val),
                                Return::Error(e) => return Err(e),
                                Return::Yield { .. } => unimplemented!("TODO: Generators"),
                            }*/
                            return C::Continue;
                        }
                    }
                }
                _ => unimplemented!("TODO: Async"),
            }
        }
        let key = self.allocate_string("call");
        if let Some(call) = match func.lookup(self, Value::from(key)) {
            Ok(x) => x,
            Err(e) => return C::Err(e),
        } {
            return match self.call(call, this, args) {
                Ok(v) => C::Ok(v),
                Err(e) => C::Err(e),
            };
        }
        return C::Err(Value::from(self.allocate_string("not a function")));
    }
    pub fn interpret(&mut self) -> Return {
        loop {
            let mut current = self.stack.current_frame();
            let bp = current.bp;
            let ip = current.ip;
            let ins = current.code.code[bp].code[ip];
            current.ip += 1;
            #[cfg(feature = "perf")]
            {
                self.perf.get_perf(ins.discriminant() as u8);
            }
            match ins {
                Ins::Mov { dst, src } => {
                    let src = current.r(src);
                    let r = current.r_mut(dst);
                    *r = src;
                }
                Ins::Yield { dst, res } => {
                    panic!();
                }
                Ins::LoadI32 { dst, imm } => {
                    *current.r_mut(dst) = Value::new_int(imm);
                }
                Ins::Safepoint => {
                    self.safepoint();
                }
                Ins::LoopHint { fdbk } => {
                    use crate::jit::types::*;
                    if self.configs.enable_jit {
                        current.code.hotness = current.code.hotness.wrapping_add(1);
                        match &mut current.clone().code.feedback[fdbk as usize] {
                            FeedBack::Loop { osr_enter, hotness } => {
                                if let Some(osr_enter) = osr_enter {
                                    if let Some(ref jit) = current.clone().code.jit_code {
                                        let osr = &jit.osr_table;
                                        let func: extern "C" fn(
                                            &mut Runtime,
                                            &mut CallFrame,
                                            usize,
                                        )
                                            -> JITResult =
                                            unsafe { std::mem::transmute(jit.instruction_start()) };
                                        match func(self, current.get_mut(), osr.labels[*osr_enter])
                                        {
                                            JITResult::Err(e) => {
                                                return Return::Error(e);
                                            }
                                            JITResult::Ok(x) => {
                                                return Return::Return(x);
                                            }
                                            _ => unimplemented!(),
                                        }
                                    }
                                } else {
                                    if *hotness >= 100 {
                                        log::trace!("Loop is hot! Doin OSR");
                                        let mut gen = FullCodegen::new(current.code.clone());
                                        gen.compile(false);
                                        let code = gen.finish(self, true);
                                        let osr = code.osr_table.labels[osr_enter.unwrap()];
                                        let mut bytecode = current.code.clone();

                                        let func: extern "C" fn(
                                            &mut Runtime,
                                            &mut CallFrame,
                                            usize,
                                        )
                                            -> JITResult = unsafe {
                                            std::mem::transmute(code.instruction_start())
                                        };
                                        bytecode.jit_code = Some(code);
                                        match func(self, current.get_mut(), osr) {
                                            JITResult::Err(e) => {
                                                return Return::Error(e);
                                            }
                                            JITResult::Ok(x) => {
                                                return Return::Return(x);
                                            }
                                            _ => unimplemented!(),
                                        }
                                    } else {
                                        *hotness += 1;
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
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
                    if let Some(x) = current.handlers.pop() {
                        current.bp = x as _;
                        current.ip = 0;
                        *current.r_mut(VirtualRegister::argument(0)) = value;
                        continue;
                    }
                    return Return::Error(value);
                }
                Ins::TryCatch { try_, catch, reg } => {
                    current.bp = try_ as _;
                    current.ip = 0;

                    current.handlers.push(catch as _);
                }
                Ins::PopCatch => {
                    current.handlers.pop();
                }
                Ins::Add { dst, lhs, src, .. } => {
                    let y = current.r(src);
                    let x = current.r(lhs);
                    if x.is_int32() && y.is_int32() {
                        if let (val, false) = x.as_int32().overflowing_add(y.as_int32()) {
                            *current.r_mut(dst) = Value::new_int(val);
                            continue;
                        }
                    }

                    *current.r_mut(dst) = Value::number(x.to_number() + y.to_number());
                }
                Ins::Sub { dst, lhs, src, .. } => {
                    let y = current.r(src);
                    let x = current.r(lhs);
                    if x.is_int32() && y.is_int32() {
                        if let (val, false) = x.as_int32().overflowing_sub(y.as_int32()) {
                            *current.r_mut(dst) = Value::new_int(val);
                            continue;
                        }
                    }
                    *current.r_mut(dst) = Value::number(x.to_number() - y.to_number());
                }
                Ins::Div { dst, lhs, src, .. } => {
                    let y = current.r(src);
                    let x = current.r(lhs);

                    *current.r_mut(dst) = Value::number(x.to_number() / y.to_number());
                }
                Ins::Mul { dst, lhs, src, .. } => {
                    let y = current.r(src);
                    let x = current.r(lhs);
                    if x.is_int32() && y.is_int32() {
                        if let (val, false) = x.as_int32().overflowing_mul(y.as_int32()) {
                            *current.r_mut(dst) = Value::new_int(val);
                            continue;
                        }
                    }
                    *current.r_mut(dst) = Value::number(x.to_number() * y.to_number());
                }
                Ins::Mod { dst, lhs, src, .. } => {
                    let y = current.r(src);
                    let x = current.r(lhs);
                    *current.r_mut(dst) = Value::number(x.to_number() % y.to_number());
                }
                Ins::Eq { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs);
                    let rhs = current.r(src);
                    let result = Self::compare_equal(lhs, rhs);
                    *current.r_mut(dst) = if result {
                        Value::true_()
                    } else {
                        Value::false_()
                    };
                }
                Ins::NEq { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs);
                    let rhs = current.r(src);
                    let result = !Self::compare_equal(lhs, rhs);
                    *current.r_mut(dst) = if result {
                        Value::true_()
                    } else {
                        Value::false_()
                    };
                }
                Ins::Greater { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs);
                    let rhs = current.r(src);
                    let result = Self::compare_greater(lhs, rhs);
                    *current.r_mut(dst) = if result {
                        Value::true_()
                    } else {
                        Value::false_()
                    };
                }
                Ins::Less { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs);
                    let rhs = current.r(src);
                    let result = Self::compare_less(lhs, rhs);
                    *current.r_mut(dst) = if result {
                        Value::true_()
                    } else {
                        Value::false_()
                    };
                }
                Ins::LessEq { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs);
                    let rhs = current.r(src);
                    let result = Self::compare_less_equal(lhs, rhs);
                    *current.r_mut(dst) = if result {
                        Value::true_()
                    } else {
                        Value::false_()
                    };
                }
                Ins::GreaterEq { dst, lhs, src, .. } => {
                    let lhs = current.r(lhs);
                    let rhs = current.r(src);
                    let result = Self::compare_greater_equal(lhs, rhs);
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
                    /*if let CellValue::Array(ref arr) = func.as_cell().value {
                        *current.r_mut(dst) = arr[up as usize];
                    } else {
                        panic!("Function does not have environment");
                    }*/
                    if let CellValue::Function(Function::Regular(ref r)) = func.as_cell().value {
                        if let CellValue::Array(ref arr) = r.env.as_cell().value {
                            *current.r_mut(dst) = arr[up as usize];
                        }
                    }
                }
                Ins::CloseEnv {
                    dst,
                    function,
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

                    let func = current.r(function);
                    match func.as_cell().value {
                        CellValue::Function(Function::Regular(ref mut r)) => {
                            let arr =
                                self.allocate_cell(Cell::new(CellValue::Array(arguments), None));
                            r.env = Value::from(arr);
                            *current.r_mut(dst) = func;
                        }
                        _ => unreachable!(),
                    }
                }
                Ins::CallNoArgs {
                    dst,
                    function,
                    this,
                } => {
                    let function = current.r(function);
                    let this = current.r(this);
                    match self.call_interp(function, this, &[]) {
                        C::Ok(val) => {
                            *current.r_mut(dst) = val;
                        }
                        C::Err(e) => return Return::Error(e),
                        C::Continue => {
                            current = self.stack.current_frame();
                            current.rreg = dst;
                            continue;
                        }
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
                    match self.call_interp(function, this, &arguments) {
                        C::Ok(val) => {
                            drop(arguments);
                            *current.r_mut(dst) = val;
                        }
                        C::Err(e) => return Return::Error(e),
                        C::Continue => {
                            drop(arguments);
                            current = self.stack.current_frame();
                            current.rreg = dst;
                            continue;
                        }
                    }
                }
                Ins::Return { val } => {
                    let val = current.r(val);
                    let r = current.rreg;
                    if current.exit_on_return {
                        self.stack.pop();
                        return Return::Return(val);
                    }
                    self.stack.pop();
                    if self.stack.stack.is_empty() {
                        return Return::Return(val);
                    }
                    current = self.stack.current_frame();
                    *current.r_mut(r) = val;
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
                Ins::NewObject { dst } => {
                    let proto = Some(self.object_prototype);
                    let cell = Cell::new(CellValue::None, proto);
                    *current.r_mut(dst) = Value::from(self.allocate_cell(cell));
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
                _ => unimplemented!("TODO: {}", ins),
            }
        }
    }
}
