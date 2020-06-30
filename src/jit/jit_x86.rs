use crate::stack::callframe::CallFrame;
use crate::value::*;
use masm::x86_assembler::*;
use masm::x86masm::*;
pub const SP: RegisterID = RegisterID::ESP;
pub const BP: RegisterID = RegisterID::EBP;
pub const REG_CALLFRAME: RegisterID = RegisterID::R14;
pub struct JIT {
    masm: MacroAssemblerX86,
}

impl JIT {
    pub fn function_prologue(&mut self, _regc: u32) {
        // this basically does push rbp;mov rbp,rsp
        self.masm.function_prologue(0); // value size is always equal to 8 bytes
    }

    pub fn function_epilogue(&mut self) {
        self.masm.function_epilogue();
    }

    pub fn call(&mut self) -> masm::Call {
        self.masm.call()
    }
}
#[cfg(target_pointer_width = "64")]
impl JIT {
    pub fn get_callframe(&mut self, to: Reg) {
        self.masm.move_rr(REG_CALLFRAME, to);
    }
    pub fn get_argument(&mut self, at: u32, to: Reg) {
        let off = offset_of!(CallFrame, arguments);
        self.masm.load64(Mem::Base(REG_CALLFRAME, off as _), to);
        self.masm.load64(Mem::Base(to, at as i32 * 8), to);
    }

    pub fn get_register(&mut self, reg: u8, to: Reg) {
        let off = offset_of!(CallFrame, registers);
        self.masm.load64(Mem::Base(REG_CALLFRAME, off as _), to);
        self.masm.load64(Mem::Base(to, at as i32 * 8), to);
    }
}
#[cfg(target_pointer_width = "32")]
impl JIT {
    pub fn get_callframe(&mut self, to: Reg) {
        self.masm.load32(Mem::Base(Reg::EBP, 0), to);
    }
    pub fn get_argument(&mut self, at: u32, tag: Reg, payload: Reg) {
        let off = offset_of!(AsBits, payload);
        let off1 = offset_of!(AsBits, tag);
        let regs = offset_of!(CallFrame, arguments);
        self.get_callframe(tag);
        self.masm.load32(Mem::Base(tag, regs as _), payload);
        self.masm.load32(Mem::Base(payload, off1 as _), tag);
        self.masm.load32(Mem::Base(payload, off as _), payload);
    }
    pub fn get_register(&mut self, at: u32, tag: Reg, payload: Reg) {
        let off = offset_of!(AsBits, payload);
        let off1 = offset_of!(AsBits, tag);
        let regs = offset_of!(CallFrame, registers);
        self.get_callframe(tag);
        self.masm.load32(Mem::Base(tag, regs as _), payload);
        self.masm.load32(Mem::Base(payload, off1 as _), tag);
        self.masm.load32(Mem::Base(payload, off as _), payload);
    }
}

pub type Reg = RegisterID;
pub type FPReg = RegisterID;

pub const T0: Reg = Reg::EAX;
pub const T1: Reg = Reg::ECX;
pub const T2: Reg = Reg::EDX;
pub const T3: Reg = Reg::EDI;
pub const T4: Reg = Reg::ESI;

pub const FT0: FPReg = FPReg::XMM0;
pub const FT1: FPReg = FPReg::XMM1;
pub const FT2: FPReg = FPReg::XMM2;
pub const FT3: FPReg = FPReg::XMM3;
pub const FT4: FPReg = FPReg::XMM4;
pub const FT5: FPReg = FPReg::XMM5;

pub const R0: Reg = Reg::EAX;
pub const R1: Reg = Reg::EDX;
