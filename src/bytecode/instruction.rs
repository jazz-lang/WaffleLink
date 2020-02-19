#[derive(Copy, Clone, PartialEq, Eq, PartialOrd)]
#[repr(u16)]
/// JLight VM instruction.
///
/// A,B,C,D - Argument numbers
/// R(_) - Register
/// B(_) - Block
/// N(_) - Number
pub enum Instruction {
    LoadNull(u16),
    LoadUndefined(u16),
    LoadInt(u16, i32),
    LoadNumber(u16, u64),
    LoadTrue(u16),
    LoadFalse(u16),
    LoadById(u16, u16, u32),
    StoreById(u16, u16, u32),
    LoadByValue(u16, u16, u16),
    StoreByValue(u16, u16, u16),
    LoadByIndex(u16, u16, u32),
    StoreByIndex(u16, u16, u32),
    LoadStaticById(u16, u32),
    StoreStaticById(u16, u32),
    LoadStaticByValue(u16, u16),
    StoreStaticByValue(u16, u16),
    /// Goto B(B) if R(A) is true, otherwise goto to B(C)
    ConditionalBranch(u16, u16, u16),
    /// Goto B(B)
    Branch(u16),
    /// Goto B(B) if R(A) is true
    BranchIfTrue(u16, u16),
    /// Goto B if R(a) is false
    BranchIfFalse(u16, u16),
    CatchBlock(
        u16, /* register to store thrown value in */
        u16, /* block index */
    ),
    /// Throw value in R(A). If there are no catch blocks then this value will cause runtime panic and show stack trace.
    Throw(u16),
    /// Initialize N(B) upvalues in function R(A)
    MakeEnv(u16, u16),
    Return(Option<u16>),
    Push(u16),
    Pop(u16),
    Call(u16, u16, u16),
    VirtCall(u16, u16, u16, u16),
    New(u16, u16, u16),
    Gc,
}
