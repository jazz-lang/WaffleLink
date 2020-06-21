#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8, C)]
pub enum Opcode {
    Null(u8),
    Constant(u8, u32),
    True(u8),
    False(u8),
    This(u8),
    SetThis(u8),
    GetEnv(u32),
    Get(u8, u8, u8),
    Set(u8, u8, u8),
    GetById(u8, u8, u32),
    SetById(u8, u8, u32),
    /// mov r(A),r(B)
    ///
    /// --> Move r(B) to r(A)
    Mov(u8, u8),
    /// set_arg r(A),arg(ix:u8)
    SetArg(u8, u8),
    /// get_arg r(A),arg(ix:u8)
    GetArg8(u8, u8),
    SetArg16(u8, u16),
    GetArg16(u8, u16),
    /// set_up r(A),arg(ix: u8)
    SetUp8(u8, u8),
    /// get_up r(A),arg(ix: u8)
    GetUp8(u8, u8),
    SetUp16(u8, u16),
    GetUp16(u8, u16),

    /// call r(A), r(B), r(C) argc: u8
    ///
    /// r(A) - return register
    /// r(B) - function to call
    /// r(C) - 'this' value
    Call8(u8, u8, u8, u8),
    /// call r(A), r(B), r(C) argc: u16
    Call16(u8, u8, u8, u8, u16),
    MakeEnv(u16),

    Add(u8, u8, u8),
    Sub(u8, u8, u8),
    Div(u8, u8, u8),
    Mul(u8, u8, u8),
    Rem(u8, u8, u8),
    Shr(u8, u8, u8),
    Shl(u8, u8, u8),

    Greater(u8, u8, u8),
    Less(u8, u8, u8),
    Eq(u8, u8, u8),
    LessEq(u8, u8, u8),
    GreaterEq(u8, u8, u8),

    Br(u16),
    BrCond(u8, u16, u16),

    Ret(u8),
}
