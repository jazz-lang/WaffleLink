#[cfg(target_arch = "x86_64")]
pub use self::x64::*;
use crate::assembler::asm_buffer::AsmBuffer;

pub mod masm;
#[cfg(target_arch = "x86_64")]
pub mod masmx64;
#[cfg(target_arch = "x86_64")]
pub mod x64;

pub mod cpu;
pub mod asm_buffer;
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Register(u8);

impl Register {
    pub fn new(value: u8) -> Register {
        Register(value)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FloatRegister(u8);

pub struct Assembler {
    code: AsmBuffer,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler { code: AsmBuffer::Vec(vec![]) }
    }

    pub fn pc(&self) -> usize {
        self.code.len()
    }

    pub fn code_mut(&mut self) -> &mut [u8] {
        self.code.as_vec_mut()
    }

    pub fn code(&self) -> &[u8] {
        self.code.as_ref()
    }

    pub fn finalize(self) -> Vec<u8> {
        self.code.as_vec()
    }

    fn emit_u8(&mut self, value: u8) {
        self.code.push(value);
    }

    fn emit_u32(&mut self, value: u32) {
        self.emit_u8(value as u8);
        self.emit_u8((value >> 8) as u8);
        self.emit_u8((value >> 16) as u8);
        self.emit_u8((value >> 24) as u8);
    }

    fn emit_u64(&mut self, value: u64) {
        self.emit_u32(value as u32);
        self.emit_u32((value >> 32) as u32);
    }
}
