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
                //std::mem::swap(&mut interp.frame, &mut frame);
                pc = interp.frame.pc;
            }
            None => return,
        }
    }
    return Interpreter::dispatch(interp, pc);
}
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
#[cfg(target = "x86_64")]
global_asm!(
    "
        enterInterpTrampoline:
            pushq %rbp
            movq %rsp,%rbp
            movq (%rax),%rcx
            addq $8,%rax
            movq %rax,32(%rdi)
            jmpq *%rcx
        
        exitInterpTrampoline: 
            movq %rbp,%rsp
            popq %rbp
            ret
    "
);
#[cfg(target = "aarch64")]
global_asm!(
    "enterInterpTrampoline: 
        str x30,[sp,#-16]!
        mov x8,x0
        mov x1,x0
        add x0,x0,#8
        br x1
    exitInterpTrampoline: 
        mov     w0, #42
        ldr     x30, [sp], #16
        ret
    "
);
impl Interpreter {
    #[inline(always)]
    pub fn dispatch(this: &mut Self, pc: Pc) {
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            this.live = true;
        }
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            let ins = this.pc.advance();
            return (ins.func())(this, pc);
        }
    }

    pub fn __run_internal(&mut self, pc: Pc) {
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            while self.live {
                Self::dispatch(self, pc);
                if self.exception.is_some() {
                    return;
                }
            }
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            /*extern "C" {
                fn enterInterpTrampoline(interp: &mut Interpreter, pc: Pc);
            }
            unsafe {
                enterInterpTrampoline(self, pc);
            }*/
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
pub static FN: fn(&mut Interpreter, Pc) = Interpreter::dispatch;
#[used]
pub static FN2: fn(&mut Interpreter, Pc) = Interpreter::__run_internal;
