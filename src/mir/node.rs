use super::opcodes::*;
/// Represents one MIR instruction
pub struct MIRNode {
    pub(super) op: Opcode,
    pub(super) inputs: Vec<Operand>,
    pub(super) outputs: Vec<Operand>,
}

#[derive(Copy, Clone)]
pub enum Operand {
    Imm32(i32),
    Imm64(i64),
    Imm16(i16),
    Imm8(i8),
    UImm64(u64),
    UImm32(u32),
    UImm16(u16),
    UImm8(u8),
    Value(u32),
}
