use super::arc::*;
use super::gc::*;
use super::opcodes::*;
use super::state::*;
use super::threads::*;
use super::value::*;
use super::*;

pub struct Interpreter {
    live: bool,
    state: Arc<AppThread>,
    error: bool,
    acc: Value,
    pc: Pc,
}

pub enum IResult {
    Continue,
    Stop,
    Ok(Value),
    Err(Value),
}

pub fn acc_stack(interp: &mut Interpreter, pc: Pc) {
    let ix = interp.pc.advance().i16();
    let val = unsafe {
        interp
            .state
            .vm_state()
            .stack
            .value_stack
            .get_unchecked(ix as usize)
    };
    interp.acc = *val;
    Interpreter::dispatch(interp, pc)
}

impl Interpreter {
    #[inline(always)]
    pub fn dispatch(this: &mut Self, pc: Pc) {
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            this.live = true;
            IResult::Continue
        }
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            let ins = this.pc.current();
            return (ins.func())(this, pc);
        }
    }
}

#[used]
pub static FOO: [fn(state: &mut super::interp::Interpreter, pc: Pc); 1] = [acc_stack];
