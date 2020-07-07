use super::*;
use masm::linkbuffer::*;
use masm::MacroAssemblerBase;
pub struct TailCallStub {
    pub fun: fn(&mut CallFrame, faddr: usize) -> WResult,
}

impl TailCallStub {
    pub fn generate(mem: &mut Memory) -> Self {
        let c = crate::bytecode::CodeBlock::new();
        let mut jit = JIT::new(&c);

        let masm = &mut jit.masm;
        #[cfg(target_arch = "x86_64")]
        {
            {
                const FADDR: Reg = Reg::ESI;
                // jmp rsi
                masm.far_jump_r(FADDR);
            }
        }
        #[cfg(target_arch = "x86")]
        {
            use masm::x86masm::Mem;
            // mov eax, dword ptr [esp + 8]
            masm.load32(Mem::Base(Reg::ESP, 8), T0);
            // jmp eax
            masm.far_jump_r(T0);
        }
        let code = jit.masm.finalize();
        let cptr = mem.allocate(code.len(), 8).unwrap();
        Self {
            fun: unsafe { std::mem::transmute(cptr) },
        }
    }
}
