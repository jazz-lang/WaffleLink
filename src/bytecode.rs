//! Register-based bytecode definition.

/// 4 byte wide opcodes.
pub enum Opcode {
    /// R(A) = R(B)
    Move(u8,u8),
    /// ACC = R(A)[R(B)]
    Load(u8,u8),
    /// ACC = FUNC.CONSTANTS[A]
    LoadConst(u16),
    /// ACC = ISOLATE.GLOBALS[R(A)]
    LoadGlobal(u8),
    /// R(A) = ACC
    LoadAcc(u8),
    /// ACC = ARG(B)
    LoadArg(u16),
    /// ACC = ACC(R(A..B))
    Call(u8,u8),
    /// return ACC
    Return,
    /// ACC = R(A)
    SetAcc(u8),
    /// ACC[R(A)] = R(B)
    Set(u8,u8),
    /// ISOLATE.GLOBALS[R(A)] = ACC
    SetGlobal(u8),
    /// Hint for JIT that we're entering loop.
    LoopHint,
    /// PC = A
    Jump(u16),
    /// if !bool(ACC) => PC = A
    JumpZero(u16),
    /// if bool(ACC) => PC = A
    JumpNZero(u16),
    /// PC = FUNC.JMPTABLE[A][int(ACC)]
    SwitchInt(u16),
    /// ACC = close(ACC,R(A)..R(C))
    MakeFunc(u8,u8),
    /// ACC = FUNC.UPVALS[A]
    LoadUpv(u16),
    /// FUNC.UPVALS[A] = ACC
    SetUpv(u16),
    /// ACC = ACC + R(A)
    Add(u8),
    /// ACC = ACC - R(A)
    Sub(u8),
    /// ACC = ACC / R(A)
    Div(u8),
    /// ACC = ACC * R(A)
    Mul(u8),
    /// ACC = ACC int_div R(A)
    IDiv(u8),
    /// ACC = ACC mod R(A)
    IMod(u8),
    /// ACC = ACC % R(A)
    Mod(u8),
    /// ACC = ACC == R(A)
    Eq(u8),
    /// ACC = ACC != R(A)
    NEq(u8),
    /// ACC = ACC > R(A)
    Gt(u8),
    /// ACC = ACC >= R(A)
    Ge(u8),
    /// ACC = ACC < R(A)
    Lt(u8),
    /// ACC = ACC <= R(A)
    Le(u8),
    /// ACC = !ACC
    Not,
    /// ACC = -ACC
    Neg,
}
const_assert!(core::mem::size_of::<Opcode>() <= 4);