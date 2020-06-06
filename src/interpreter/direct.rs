//! Direct threaded interpreter. (To enable use Config::default().fast_interp()).
//!
//!
//! Direct threaded interpreter does not have dispatch loop so CPU can predict branches properly
//! this interpreter just does `goto *pc++` which is fast,but sadly requires unsafe if we want to make it work fast.
//!

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
pub struct DirectThreaded<'a> {
    live: bool,
    rt: &'a mut Runtime,
}

impl<'a> DirectThreaded<'a> {
    pub unsafe fn dispatch(&mut self, current: deref_ptr::DerefPointer<CallFrame>, mut pc: Pc) {
        // 1) target = pc++;
        let func = pc.advance();
        // 2) goto *target;
        (func.func)(self, current, pc)
    }
}

use deref_ptr::*;
pub unsafe extern "C" fn mov(i: &mut DirectThreaded, mut c: DerefPointer<CallFrame>, mut pc: Pc) {
    let [dst, src] = pc.advance().reg2;
    let src = c.r(src);
    let r = c.r_mut(dst);
    *r = src;
    i.dispatch(c, pc)
}
