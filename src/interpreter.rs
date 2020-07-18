pub mod callframe;
pub mod register;
pub mod stack_alignment;
use crate::*;
use bytecode::*;
use value::*;
pub extern "C" fn interp_loop(callframe: &mut callframe::CallFrame) -> WaffleResult {
    let cb = callframe.code_block.unwrap();
    let code = &cb.instructions;
    let mut pc = callframe.pc;
    loop {
        let ins = unsafe { *code.get_unchecked(pc as usize) };
        pc += 1;
        match ins {
            Ins::Return(value) => {
                let val = callframe.get_register(value);
                return WaffleResult::okay(val);
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
            _ => todo!(),
        }
    }
}
