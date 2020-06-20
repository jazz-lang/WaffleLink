#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8, C)]
pub enum Opcode {
    Null,
    Constant,
    True,
    False,
    This,
    SetThis,
    GetEnv,
    Get,
    Set,
    GetById,
    SetById,
    /// mov r(A),r(B)
    ///
    /// --> Move r(B) to r(A)
    Mov,
    /// set_arg r(A),arg(ix:u8)
    SetArg8,
    /// get_arg r(A),arg(ix:u8)
    GetArg8,
    SetArg16,
    GetArg16,
    /// set_up r(A),arg(ix: u8)
    SetUp8,
    /// get_up r(A),arg(ix: u8)
    GetUp8,
    SetUp16,
    GetUp16,

    /// call r(A), r(B), r(C) argc: u8
    ///
    /// r(A) - return register
    /// r(B) - function to call
    /// r(C) - 'this' value
    Call8,
    /// call r(A), r(B), r(C) argc: u16
    Call16,

    MakeEnv,

    Add,
    Sub,
    Div,
    Mul,
    Rem,
    Shr,
    Shl,
    Ret,
}
