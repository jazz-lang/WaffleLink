pub mod callframe;
pub mod register;
pub mod stack_alignment;
use crate::*;
use bytecode::*;
use jit::operations::*;
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
    let update_pc = |pc: &mut u32, off: i32| {
        *pc = (*pc as i32 + off) as u32;
    };
    loop {
        let ins = unsafe { *code.get_unchecked(pc as usize) };
        pc += 1;
        match ins {
            Ins::LoopHint => {
                cb.exc_counter = cb.exc_counter.wrapping_add(10);
                if cb.exc_counter >= crate::get_vm().jit_threshold {
                    use crate::jit::*;
                    let mut jit = JIT::new(&cb);
                    jit.compile_without_linking();
                    jit.link();
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
            Ins::Return(value) => {
                let val = callframe.get_register(value);
                return WaffleResult::okay(val);
            }
            Ins::Equal(dst, x, y) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_eq,==);
                callframe.put_register(dst, Value::new_bool(res));
            }
            Ins::NotEqual(dst, x, y) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_neq,!=);
                callframe.put_register(dst, Value::new_bool(res));
            }
            Ins::Less(dst, x, y) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_less,<);
                callframe.put_register(dst, Value::new_bool(res));
            }
            Ins::LessOrEqual(dst, x, y) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_lesseq,<=);
                callframe.put_register(dst, Value::new_bool(res));
            }
            Ins::Jmp(off) => {
                pc = (pc as i32 + off) as u32;
            }
            Ins::JmpIfNotZero(x, off) => {
                let val = callframe.get_register(x);
                if val.to_boolean() {
                    pc = (pc as i32 + off) as u32;
                }
            }
            Ins::JmpIfZero(x, off) => {
                let val = callframe.get_register(x);
                if !val.to_boolean() {
                    pc = (pc as i32 + off) as u32;
                }
            }
            Ins::JLess(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_less,<);
                if res {
                    update_pc(&mut pc, target);
                }
            }
            Ins::JLessEq(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_lesseq,<=);
                if res {
                    update_pc(&mut pc, target);
                }
            }
            Ins::JGreater(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_greater,>);
                if res {
                    update_pc(&mut pc, target);
                }
            }
            Ins::JGreaterEq(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_greaterq,>=);
                if res {
                    update_pc(&mut pc, target);
                }
            }
            Ins::JNLess(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_greater,>);
                if res {
                    update_pc(&mut pc, target);
                }
            }
            Ins::JNLessEq(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_greaterq,>=);
                if res {
                    update_pc(&mut pc, target);
                }
            }
            Ins::JNGreater(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_less,<);
                if res {
                    update_pc(&mut pc, target);
                }
            }
            Ins::JNGreaterEq(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_lesseq,<=);
                if res {
                    update_pc(&mut pc, target);
                }
            }
            Ins::JEq(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_eq,==);
                if res {
                    update_pc(&mut pc, target);
                }
            }
            Ins::JNEq(x, y, target) => {
                let x = callframe.get_register(x);
                let y = callframe.get_register(y);
                let res = cmp!(x,y,operation_compare_neq,!=);
                if res {
                    update_pc(&mut pc, target);
                }
            }
            Ins::Try(h) => {
                callframe.handlers.push(pc + h);
            }
            Ins::TryEnd => {
                callframe.handlers.pop().unwrap();
            }
            Ins::Catch(dst) => {
                let exc = crate::get_vm().exception;
                callframe.put_register(dst, exc);
            }
            Ins::Move(dst, src) => {
                let r = callframe.get_register(src);
                callframe.put_register(dst, r);
            }
            Ins::Call(dest, this, callee, argc) => {
                let this = callframe.get_register(this);
                let callee = callframe.get_register(callee);
                let result =
                    crate::jit::operations::operation_call_func(callframe, callee, argc, this);
                if result.is_okay() {
                    callframe.put_register(dest, result.value());
                } else {
                    catch!(result.value());
                }
            }
            _ => todo!(),
        }
    }
}
