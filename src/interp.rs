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
    frame: Frame,
    exception: Option<Value>,
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
            .frame.stack
            .get_unchecked(ix as usize)
    };
    interp.frame.acc = *val;
    Interpreter::dispatch(interp, pc)
}

pub fn ret(interp: &mut Interpreter,pc: Pc) {
    if interp.frame.exit_on_return {
        return;
    } else {
        match interp.state.vm_state().stack.pop() {
            Some(mut frame) => {
                std::mem::replace(&mut interp.frame, frame);
                let pc = interp.frame.pc;
                return Interpreter::dispatch(interp, pc);
            }
            None => return,
        }
    }
}

pub fn jump_if(interp: &mut Interpreter,mut pc: Pc) {
    let [if_true,if_false] = pc.advance().jump();
    let val = interp.frame.acc;
    if val.to_boolean() {
        interp.frame.pc = Pc::new(&interp.frame.code.get()[if_true as usize]);
    } else {
        interp.frame.pc = Pc::new(&interp.frame.code.get()[if_false as usize]);
    }
    Interpreter::dispatch(interp, interp.frame.pc);
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
            let ins = this.pc.advance();
            return (ins.func())(this, pc);
        }
    }

    pub fn run(&mut self,pc: Pc) -> Result<Value,Value> {
        Self::dispatch(self, pc);
        match self.exception.take() {
            Some(exc) => Err(exc),
            None => Ok(self.frame.acc)
        }
    }
}

#[used]
pub static FOO: [fn(state: &mut super::interp::Interpreter, pc: Pc); 1] = [acc_stack];
