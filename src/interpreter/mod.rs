pub mod callstack;
use crate::bytecode::*;
use crate::runtime;
use callstack::*;
use cell::*;
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
    pub async fn interpret(&mut self) -> Return {
        let mut current = self.stack.current_frame();
        'interp: loop {
            let bp = current.bp;
            let ip = current.ip;
            let ins = current.code.code[bp].code[ip];
            match ins {
                Ins::Mov { dst, src } => {
                    let src = current.r(src);
                    let r = current.r_mut(dst);
                    *r = src;
                }
                Ins::Return { val } => {
                    let val = current.r(val);
                    if current.exit_on_return || self.stack.is_empty() {
                        return Return::Return(val);
                    }
                    self.stack.pop();
                    current = self.stack.current_frame();
                }
                Ins::Yield { dst, res } => {
                    return Return::Yield(dst, current.r(res));
                }
                Ins::Await { dst, on } => {
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
                }
                _ => unimplemented!("TODO!"),
            }
        }
    }
}
