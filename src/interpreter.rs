pub mod callframe;
pub mod register;
pub mod stack_alignment;
use crate::*;
use bytecode::*;
use jit::operations::*;
use object::*;
use value::*;

macro_rules! cmp {
    ($x: expr,$y: expr,$slow: expr,$op: tt) => {{
        let result;
        {
            if $x.is_int32() && $y.is_int32( ){
                result = ($x.to_int32()) $op ($y.to_int32());
            } else {
                let fun: extern "C" fn(Value,Value) -> bool = $slow;
                result = fun($x,$y);
            }
        };
        result
    }
    };
}

pub extern "C" fn interp_loop(callframe: &mut callframe::CallFrame) -> WaffleResult {
    let mut cb = callframe.code_block.unwrap();
    let code: &Vec<Ins> = unsafe { &*(&cb.instructions as *const _) };
    let mut pc = callframe.pc;
    let vm = crate::get_vm();
    macro_rules! catch {
        ($val: expr) => {
            if let Some(handler) = callframe.handlers.pop() {
                pc = handler;
                crate::get_vm().exception = $val;
            } else {
                return WaffleResult::error($val);
            }
        };
    }
    macro_rules! binop {
        ($x: expr,$y: expr,$slow: expr,$int_op: ident,$op: tt) => {
            if $x.is_int32() && $y.is_int32() {
                let xi = $x.to_int32();
                let yi = $y.to_int32();

                let res = xi.$int_op(yi);
                if res.1 {
                    Value::new_double(xi as f64 $op yi as f64)
                } else {
                    Value::new_int(res.0)
                }
            } else if $x.is_number() && $y.is_number() {
                Value::new_double($x.to_number() $op $y.to_number())
            } else {
                let slow = $slow;
                slow(vm,$x,$y)
            }
        };
    }
    let update_pc = |pc: &mut u32, off: i32| {
        *pc = (*pc as i32 + off) as u32;
    };
    loop {
        //let mut b = String::new();
        //cb.dump_ins(&mut b, pc as _).unwrap();
        //println!("[{:4}] {}", pc, b);
        let ins = cb.instructions[pc as usize];
        match ins {
            Ins::LoopHint => {
                if vm.template_jit {
                    cb.exc_counter = cb.exc_counter.wrapping_add(1);
                    if cb.exc_counter >= crate::get_vm().jit_threshold {
                        use crate::jit::*;
                        log!("Triggering OSR after ~{} loop iterations", cb.exc_counter);
                        let mut jit = JIT::new(&cb);
                        jit.compile_without_linking();
                        jit.link();
                        if vm.disasm {
                            jit.disasm();
                        }
                        let addr = cb.jit_data().code_map.get(&(pc - 1)).copied().unwrap();
                        let trampoline = crate::get_vm()
                            .stubs
                            .get_stub(thunk_generator::osr_from_interpreter_to_jit_generator);
                        let trampoline_fn: extern "C" fn(
                            &mut callframe::CallFrame,
                            *const u8,
                        ) -> WaffleResult = unsafe { std::mem::transmute(trampoline) };
                        // TemplateJIT can't do OSR exit to interpreter, OptimizingJIT does OSR exit to template JIT.
                        return trampoline_fn(callframe, addr);
                    }
                }
                pc += 1;
            }
            Ins::Return(value) => {
                let val = callframe.get_register(value);
                return WaffleResult::okay(val);
            }
            Ins::LoadGlobal(dest, key) => {
                debug_assert!(cb.constants[key as usize].as_cell().is_string());
                let constant = cb
                    .get_constant(virtual_register::VirtualRegister::new_constant_index(
                        key as _,
                    ))
                    .as_cell()
                    .cast::<WaffleString>();
                let global = vm.globals.lookup(constant.str());
                if let Some(global) = global {
                    callframe.put_register(dest, global);
                } else {
                    if let Some(m) = callframe
                        .callee
                        .as_cell()
                        .cast::<function::Function>()
                        .module
                    {
                        if let Some(g) = m.scope.get(constant.str()) {
                            callframe.put_register(dest, *g);
                            pc += 1;
                            continue;
                        }
                    }
                    catch!(Value::from(
                        WaffleString::new(
                            &mut vm.heap,
                            &format!("global '{}' not found", constant.str())
                        )
                        .cast()
                    ));
                }
                pc += 1;
            }
            Ins::StoreGlobal(src, key) => {
                debug_assert!(cb.constants[key as usize].as_cell().is_string());
                let constant = cb
                    .get_constant(virtual_register::VirtualRegister::new_constant_index(
                        key as _,
                    ))
                    .as_cell()
                    .cast::<WaffleString>();
                let val = callframe.get_register(src);
                if vm.globals.has(constant.str()) {
                    vm.globals.insert(constant.str(), val);
                } else {
                    if let Some(mut m) = callframe
                        .callee
                        .as_cell()
                        .cast::<function::Function>()
                        .module
                    {
                        if let Some(_) = m.scope.get(constant.str()) {
                            m.scope.insert(constant.str().to_owned(), val);
                            pc += 1;
                            continue;
                        }
                    }
                    catch!(Value::from(
                        WaffleString::new(
                            &mut vm.heap,
                            &format!("global '{}' not found", constant.str())
                        )
                        .cast()
                    ));
                }
                pc += 1;
            }
            Ins::Mod(dest, lhs, rhs) => {
                let lhs = callframe.get_register(lhs);
                let rhs = callframe.get_register(rhs);
                let res = operation_value_mod(vm, lhs, rhs);
                callframe.put_register(dest, res);
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(lhs, rhs);
                pc += 1;
            }
            Ins::Add(dest, lhs, rhs) => {
                let lhs = callframe.get_register(lhs);
                let rhs = callframe.get_register(rhs);
                let res = binop!(lhs,rhs,operation_value_add,overflowing_add,+);
                callframe.put_register(dest, res);
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(lhs, rhs);
                pc += 1;
            }
            Ins::Sub(dest, lhs, rhs) => {
                let lhs = callframe.get_register(lhs);
                let rhs = callframe.get_register(rhs);
                let res = binop!(lhs,rhs,operation_value_sub,overflowing_sub,-);
                callframe.put_register(dest, res);
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(lhs, rhs);
                pc += 1;
            }
            Ins::Mul(dest, lhs, rhs) => {
                let lhs = callframe.get_register(lhs);
                let rhs = callframe.get_register(rhs);
                let res = binop!(lhs,rhs,operation_value_mul,overflowing_mul,*);
                callframe.put_register(dest, res);
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(lhs, rhs);
                pc += 1;
            }
            Ins::Div(dest, lhs, rhs) => {
                let lhs = callframe.get_register(lhs);
                let rhs = callframe.get_register(rhs);
                fn div(x: Value, y: Value) -> Value {
                    if x.is_number() && y.is_number() {
                        let res = x.to_number() / y.to_number();
                        if res as i32 as f64 == res {
                            return Value::new_int(res as _);
                        } else {
                            return Value::new_double(res);
                        }
                    }
                    Value::new_double(std::f64::NAN)
                }
                callframe.put_register(dest, div(lhs, rhs));
                pc += 1;
            }
            Ins::LShift(dest, lhs, rhs) => {
                let lhs = callframe.get_register(lhs);
                let rhs = callframe.get_register(rhs);
                if lhs.is_number() && rhs.is_number() {
                    if lhs.is_int32() && rhs.is_int32() {
                        callframe
                            .put_register(dest, Value::new_int(lhs.to_int32() << rhs.to_int32()));
                    } else {
                        callframe.put_register(
                            dest,
                            Value::new_int(
                                (lhs.to_number().trunc() as i32) << rhs.to_number().trunc() as i32,
                            ),
                        );
                    }
                } else {
                    callframe.put_register(dest, Value::undefined());
                }
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(lhs, rhs);
                pc += 1;
            }
            Ins::RShift(dest, lhs, rhs) => {
                let lhs = callframe.get_register(lhs);
                let rhs = callframe.get_register(rhs);
                if lhs.is_number() && rhs.is_number() {
                    if lhs.is_int32() && rhs.is_int32() {
                        callframe
                            .put_register(dest, Value::new_int(lhs.to_int32() >> rhs.to_int32()));
                    } else {
                        callframe.put_register(
                            dest,
                            Value::new_int(
                                (lhs.to_number().trunc() as i32) >> rhs.to_number().trunc() as i32,
                            ),
                        );
                    }
                } else {
                    callframe.put_register(dest, Value::undefined());
                }
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(lhs, rhs);
                pc += 1;
            }
            Ins::URShift(dest, lhs, rhs) => {
                let lhs = callframe.get_register(lhs);
                let rhs = callframe.get_register(rhs);
                if lhs.is_number() && rhs.is_number() {
                    if lhs.is_int32() && rhs.is_int32() {
                        callframe.put_register(
                            dest,
                            Value::new_int((lhs.to_uint32() >> rhs.to_uint32()) as i32),
                        );
                    } else {
                        callframe.put_register(
                            dest,
                            Value::new_int(
                                ((lhs.to_number().trunc() as i32 as u32)
                                    >> rhs.to_number().trunc() as i32 as u32)
                                    as i32,
                            ),
                        );
                    }
                } else {
                    callframe.put_register(dest, Value::undefined());
                }
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(lhs, rhs);
                pc += 1;
            }
            Ins::BitAnd(dest, lhs, rhs) => {
                let lhs = callframe.get_register(lhs);
                let rhs = callframe.get_register(rhs);
                if lhs.is_number() && rhs.is_number() {
                    if lhs.is_int32() && rhs.is_int32() {
                        callframe.put_register(
                            dest,
                            Value::new_int((lhs.to_uint32() & rhs.to_uint32()) as i32),
                        );
                    } else {
                        callframe.put_register(
                            dest,
                            Value::new_int(
                                ((lhs.to_number().trunc() as i32 as u32)
                                    & rhs.to_number().trunc() as i32 as u32)
                                    as i32,
                            ),
                        );
                    }
                } else {
                    callframe.put_register(dest, Value::undefined());
                }
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(lhs, rhs);
                pc += 1;
            }
            Ins::BitOr(dest, lhs, rhs) => {
                let lhs = callframe.get_register(lhs);
                let rhs = callframe.get_register(rhs);
                if lhs.is_number() && rhs.is_number() {
                    if lhs.is_int32() && rhs.is_int32() {
                        callframe.put_register(
                            dest,
                            Value::new_int((lhs.to_uint32() ^ rhs.to_uint32()) as i32),
                        );
                    } else {
                        callframe.put_register(
                            dest,
                            Value::new_int(
                                ((lhs.to_number().trunc() as i32 as u32)
                                    ^ rhs.to_number().trunc() as i32 as u32)
                                    as i32,
                            ),
                        );
                    }
                } else {
                    callframe.put_register(dest, Value::undefined());
                }
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(lhs, rhs);
                pc += 1;
            }
            Ins::BitXor(dest, lhs, rhs) => {
                let lhs = callframe.get_register(lhs);
                let rhs = callframe.get_register(rhs);
                if lhs.is_number() && rhs.is_number() {
                    if lhs.is_int32() && rhs.is_int32() {
                        callframe.put_register(
                            dest,
                            Value::new_int((lhs.to_uint32() ^ rhs.to_uint32()) as i32),
                        );
                    } else {
                        callframe.put_register(
                            dest,
                            Value::new_int(
                                ((lhs.to_number().trunc() as i32 as u32)
                                    ^ rhs.to_number().trunc() as i32 as u32)
                                    as i32,
                            ),
                        );
                    }
                } else {
                    callframe.put_register(dest, Value::undefined());
                }
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(lhs, rhs);
                pc += 1;
            }
            Ins::ToBoolean(dest, source) => {
                let src = callframe.get_register(source);
                let boolean = src.to_boolean();
                callframe.put_register(dest, Value::new_bool(boolean));
                pc += 1;
            }
            Ins::Equal(dst, x, y) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_eq,==);
                callframe.put_register(dst, Value::new_bool(res));
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(x, y);
                pc += 1;
            }
            Ins::NotEqual(dst, x, y) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_neq,!=);
                callframe.put_register(dst, Value::new_bool(res));
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(x, y);
                pc += 1;
            }
            Ins::Less(dst, x, y) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_less,<);
                callframe.put_register(dst, Value::new_bool(res));
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(x, y);
                pc += 1;
            }
            Ins::LessOrEqual(dst, x, y) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_lesseq,<=);
                callframe.put_register(dst, Value::new_bool(res));
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(x, y);
                pc += 1;
            }
            Ins::Greater(dst, x, y) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_greater,>);
                //println!("Less? {}", res);
                callframe.put_register(dst, Value::new_bool(res));
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(x, y);
                pc += 1;
            }
            Ins::GreaterOrEqual(dst, x, y) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_greatereq,>=);
                callframe.put_register(dst, Value::new_bool(res));
                callframe.code_block.unwrap().metadata[pc as usize]
                    .arith_profile
                    .observe_lhs_and_rhs(x, y);
                pc += 1;
            }
            Ins::Jmp(off) => {
                ////pc-=1;
                pc = (pc as i32 + off) as u32;
            }
            Ins::JmpIfNotZero(x, off) => {
                let val = callframe.get_register(x);
                if val.to_boolean() {
                    pc = (pc as i32 + off) as u32;
                } else {
                    pc += 1;
                }
            }
            Ins::JmpIfZero(x, off) => {
                let val = callframe.get_register(x);
                if !val.to_boolean() {
                    //pc-=1;
                    pc = (pc as i32 + off) as u32;
                } else {
                    pc += 1;
                }
            }
            Ins::JLess(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_less,<);
                if res {
                    //pc-=1;
                    update_pc(&mut pc, target);
                } else {
                    pc += 1;
                }
            }
            Ins::JLessEq(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_lesseq,<=);
                if res {
                    //pc-=1;
                    update_pc(&mut pc, target);
                } else {
                    pc += 1;
                }
            }
            Ins::JGreater(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_greater,>);
                if res {
                    //pc-=1;
                    update_pc(&mut pc, target);
                } else {
                    pc += 1;
                }
            }
            Ins::JGreaterEq(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_greatereq,>=);
                if res {
                    //pc-=1;
                    update_pc(&mut pc, target);
                } else {
                    pc += 1;
                }
            }
            Ins::JNLess(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_greater,>);
                if res {
                    //pc-=1;
                    update_pc(&mut pc, target);
                } else {
                    pc += 1;
                }
            }
            Ins::JNLessEq(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_greatereq,>=);
                if res {
                    //pc-=1;
                    update_pc(&mut pc, target);
                } else {
                    pc += 1;
                }
            }
            Ins::JNGreater(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_less,<);
                if res {
                    //pc-=1;
                    update_pc(&mut pc, target);
                } else {
                    pc += 1;
                }
            }
            Ins::JNGreaterEq(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_lesseq,<=);
                if res {
                    //pc-=1;
                    update_pc(&mut pc, target);
                } else {
                    pc += 1;
                }
            }
            Ins::JEq(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_eq,==);
                if res {
                    //pc-=1;
                    update_pc(&mut pc, target);
                } else {
                    pc += 1;
                }
            }
            Ins::JNEq(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_neq,!=);
                if res {
                    //pc-=1;
                    update_pc(&mut pc, target);
                } else {
                    pc += 1;
                }
            }
            Ins::Try(h) => {
                callframe.handlers.push(pc + h);
                pc += 1;
            }
            Ins::TryEnd => {
                callframe.handlers.pop().unwrap();
                pc += 1;
            }
            Ins::LoadId(dest, object, key) => {
                let key = cb.constants[key as usize];
                let object = callframe.get_register(object);
                let res = operation_get_by(vm, object, key);
                if res.is_error() {
                    catch!(res.value());
                }
                callframe.put_register(dest, res.value());
                pc += 1;
            }
            Ins::StoreId(object, id, val) => {
                let key = cb.constants[id as usize];
                let object = callframe.get_register(object);
                let val = callframe.get_register(val);
                let res = operation_put_by(vm, object, key, val);
                if res.is_error() {
                    catch!(res.value());
                }
                pc += 1;
            }
            Ins::Store(object, key, val) => {
                let key = callframe.get_register(key);
                let object = callframe.get_register(object);
                let val = callframe.get_register(val);
                let res = operation_put_by(vm, object, key, val);
                if res.is_error() {
                    catch!(res.value());
                }
                pc += 1;
            }
            Ins::Load(dest, object, key) => {
                let key = callframe.get_register(key);
                let object = callframe.get_register(object);
                let res = operation_get_by(vm, object, key);
                if res.is_error() {
                    catch!(res.value());
                }
                callframe.put_register(dest, res.value());
                pc += 1;
            }
            Ins::LoadU(dest, idx) => {
                if let Some(env) = callframe.callee.as_cell().cast::<function::Function>().env {
                    let val = env.get_at(idx as _);
                    callframe.put_register(dest, val);
                } else {
                    catch!(Value::from(
                        WaffleString::new(&mut vm.heap, "can't load upvalue, no environment found")
                            .cast()
                    ))
                }
                pc += 1;
            }
            Ins::StoreU(src, idx) => {
                if let Some(mut env) = callframe.callee.as_cell().cast::<function::Function>().env {
                    let val = callframe.get_register(src);
                    env.set_at(idx as _, val);
                } else {
                    catch!(Value::from(
                        WaffleString::new(
                            &mut vm.heap,
                            "can't store upvalue, no environment found"
                        )
                        .cast()
                    ))
                }
                pc += 1;
            }
            Ins::Closure(f, count) => {
                let func = callframe.get_register(f);
                if func.is_cell() {
                    debug_assert!(func.as_cell().is_function());
                    let mut func = func.as_cell().cast::<function::Function>();
                    let values = &callframe.regs
                        [f.to_local() as usize + 1..f.to_local() as usize + count as usize];
                    let mut array = Array::new(&mut vm.heap, values.len(), Value::undefined());
                    for (i, val) in values.iter().enumerate() {
                        array.set_at(i, *val);
                    }
                    func.env = Some(array);
                } else {
                    unreachable!();
                }
                pc += 1;
            }
            Ins::Catch(dst) => {
                let exc = crate::get_vm().exception;
                callframe.put_register(dst, exc);
                pc += 1;
            }
            Ins::Move(dst, src) => {
                let r = callframe.get_register(src);
                callframe.put_register(dst, r);
                pc += 1;
            }
            Ins::Call(dest, this, callee_r, argc) => {
                let this = callframe.get_register(this);
                let callee = callframe.get_register(callee_r);

                let result = crate::jit::operations::operation_call_func(
                    callframe, callee, callee_r, argc, this,
                );

                if result.is_okay() {
                    callframe.put_register(dest, result.value());
                } else {
                    catch!(result.value());
                }
                pc += 1;
            }
            Ins::NewObject(dst) => {
                let object = object::RegularObj::new(&mut vm.heap, Value::undefined());
                callframe.put_register(dst, Value::from(object.cast::<Obj>()));
                pc += 1;
            }
            Ins::New(dest, callee_r, argc) => {
                let callee = callframe.get_register(callee_r);
                if callee.is_cell() {
                    if let Some(lookup) = callee.as_cell().vtable.lookup_fn {
                        let ctor = lookup(vm, callee.as_cell(), vm.constructor);
                        let proto = lookup(vm, callee.as_cell(), vm.prototype);
                        if proto.is_error() {
                            catch!(ctor.value());
                        }
                        let proto = if proto.value().is_cell() {
                            proto.value()
                        } else {
                            Value::undefined()
                        };
                        if ctor.is_error() {
                            catch!(ctor.value());
                        } else if ctor.value().is_cell() && ctor.value().as_cell().is_function() {
                            let this = RegularObj::new(&mut vm.heap, proto);
                            let result = crate::jit::operations::operation_call_func(
                                callframe,
                                ctor.value(),
                                callee_r,
                                argc,
                                Value::from(this.cast()),
                            );
                            if result.is_okay() {
                                callframe.put_register(dest, Value::from(this.cast()));
                            } else {
                                catch!(result.value());
                            }
                        } else {
                            catch!(Value::from(
                                WaffleString::new(&mut vm.heap, "constructor is not a function!")
                                    .cast()
                            ));
                        }
                    } else {
                        catch!(Value::from(
                            WaffleString::new(&mut vm.heap, "Can't find constructor").cast()
                        ));
                    }
                } else {
                    catch!(Value::from(
                        WaffleString::new(&mut vm.heap, "callee is not an object/function!").cast()
                    ));
                }
                pc += 1;
            }
            Ins::Safepoint => {
                if vm.stop_world {
                    let sp = false;
                    vm.heap.collect(gc::Address::from_ptr(&sp));
                }
                pc += 1;
            }
            Ins::LoadThis(dest) => {
                let val = callframe.this;
                callframe.put_register(dest, val);
                pc += 1;
            }
            Ins::Neg(dest, src) => {
                let src = callframe.get_register(src);
                if src.is_number() {
                    if src.is_int32() {
                        callframe.put_register(dest, Value::new_int(-src.as_int32()));
                    } else {
                        callframe.put_register(dest, Value::new_double(-src.as_double()));
                    }
                } else {
                    callframe.put_register(dest, Value::new_double(pure_nan::pure_nan()));
                }
                pc += 1;
            }
            Ins::Not(dest, src) => {
                let src = callframe.get_register(src).to_boolean();
                callframe.put_register(dest, Value::new_bool(!src));
                pc += 1;
            }
            x => todo!("NYI {}", x),
        }
    }
}
