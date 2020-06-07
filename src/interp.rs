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
    frame: Handle<Frame>,
    exception: Option<Value>,
    pc: Pc,
}

pub enum IResult {
    Continue,
    Stop,
    Ok(Value),
    Err(Value),
}

#[naked]
pub fn acc_null(interp: &mut Interpreter,pc: Pc) {
    interp.frame.acc = Value::null();
    Interpreter::dispatch(interp, pc)
}

#[naked]
pub fn acc_true(interp: &mut Interpreter,pc: Pc) {
    interp.frame.acc = Value::true_();
    Interpreter::dispatch(interp, pc);
} 

#[naked]
pub fn acc_false(interp: &mut Interpreter,pc: Pc) {
    interp.frame.acc = Value::false_();
    Interpreter::dispatch(interp, pc);
}

#[naked]
pub fn acc_this(interp: &mut Interpreter,pc: Pc) {
    interp.frame.acc = interp.frame.this;
    Interpreter::dispatch(interp, pc);
}
#[naked]
pub fn acc_int(interp: &mut Interpreter,mut pc: Pc) {
    interp.frame.acc = Value::new_int(pc.advance().i32());
    Interpreter::dispatch(interp, pc);
}

#[naked]
pub fn acc_stack(interp: &mut Interpreter, pc: Pc) {
    let ix = interp.pc.advance().i16();
    let val = unsafe { interp.frame.stack.get_unchecked(ix as usize) };
    interp.frame.acc = *val;
    Interpreter::dispatch(interp, pc)
}

#[optimize(speed)]
#[naked]
pub fn ret_interp(interp: &mut Interpreter, _: Pc) {
    let pc;
    if interp.frame.exit_on_return {
        return;
    } else {
        match interp.state.vm_state().stack.pop() {
            Some(mut frame) => {
                interp.frame = frame;
                pc = interp.frame.pc;
            }
            None => return,
        }
    }
    return Interpreter::dispatch(interp, pc);
}
#[optimize(speed)]
#[naked]
pub fn jump_if(interp: &mut Interpreter, mut pc: Pc) {
    unsafe {
        let [if_true, if_false] = pc.advance().jump();
        let val = interp.frame.acc;
        if val.to_boolean() {
            interp.frame.pc = Pc::new(&*interp.frame.code.get().get_unchecked(if_true as usize));
        } else {
            interp.frame.pc = Pc::new(&*interp.frame.code.get().get_unchecked(if_false as usize));
        }
        Interpreter::dispatch(interp, interp.frame.pc);
    }
}

impl Interpreter {
    #[optimize(size)]
    #[inline(always)]
    pub extern "C" fn dispatch(this: &mut Self /* rdi */, mut pc: Pc /* rsi*/) {
        #[cfg(debug_assertions)]
        {
            this.live = true;
        }
        #[cfg(all(
            any(
                target_arch="powerpc64",
                target_arch="mips64",
                target_arch = "x86_64",
                target_arch="arm",
                target_arch = "aarch64",
                not(target_arch = "x86"), // LLVM does not do TCO for these archs
                not(target_arch = "mips"),
                not(target_arch = "powerpc"),
                not(target_arch="powerpc64")
            ),
            not(debug_assertions)
        ))]
        {
            let ins = pc.advance();
            return (ins.func())(this, pc);
        }
    }

    pub fn __run_internal(&mut self, pc: Pc) {
        #[cfg(debug_assertions)]
        {
            while self.live {
                Self::dispatch(self, pc);
                if self.exception.is_some() {
                    return;
                }
            }
        }

        #[cfg(all(
            any(
                target_arch="powerpc64",
                target_arch="mips64",
                target_arch = "x86_64",
                target_arch="arm",
                target_arch = "aarch64",
                not(target_arch = "x86"), // LLVM does not do TCO for these archs
                not(target_arch = "mips"),
                not(target_arch = "powerpc"),
                not(target_arch="powerpc64")
            ),
            not(debug_assertions)
        ))]
        {
            Self::dispatch(self, pc);
        }
    }

    pub fn run(&mut self, pc: Pc) -> Result<Value, Value> {
        self.__run_internal(pc);
        match self.exception.take() {
            Some(exc) => Err(exc),
            None => Ok(self.frame.acc),
        }
    }
}

#[used]
pub static FOO: [fn(state: &mut super::interp::Interpreter, pc: Pc); 2] = [acc_stack, ret_interp];

#[used]
pub static FN: extern "C" fn(&mut Interpreter, Pc) = Interpreter::dispatch;
#[used]
pub static FN2: fn(&mut Interpreter, Pc) = Interpreter::__run_internal;
