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
    StoreStack(u16, u32),
    LoadStack(u16, u32),
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
    TailCall(u16, u16, u16),
    New(u16, u16, u16),
    /// Triggers full GC cycle.
    Gc,
    /// Safepoint for GC. This instruction should be placed in `SafepointPass` or by your compiler.
    GcSafepoint,
    Binary(BinOp, u16, u16, u16),
    Unary(UnaryOp, u16, u16),
    Move(u16, u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinOp {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
    Rsh,
    Lsh,

    Greater,
    Less,
    LessOrEqual,
    GreaterOrEqual,
    Equal,
    NotEqual,
    And,
    Or,
    Xor,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[allow(non_snake_case)]
pub mod InstructionByte {
    pub const LOAD_NULL: u8 = 0x0;
    pub const LOAD_UNDEF: u8 = 0x1;
    pub const LOAD_INT: u8 = 0x2;
    pub const LOAD_NUM: u8 = 0x3;
    pub const LOAD_TRUE: u8 = 0x4;
    pub const LOAD_FALSE: u8 = 0x5;
    pub const LOAD_BY_ID: u8 = 0x6;
    pub const STORE_BY_ID: u8 = 0x7;
    pub const LOAD_BY_VALUE: u8 = 0x8;
    pub const STORE_BY_VALUE: u8 = 0x9;
    pub const LOAD_BY_INDEX: u8 = 0xa;
    pub const STORE_BY_INDEX: u8 = 0xb;
    pub const LOAD_STATIC_BY_ID: u8 = 0xc;
    pub const STORE_STATIC_BY_ID: u8 = 0xd;
    pub const LOAD_STATIC_BY_VALUE: u8 = 0xe;
    pub const STORE_STACK: u8 = 0xf;
    pub const LOAD_STACK: u8 = 0x10;
    pub const CONDITIONAL_BRANCH: u8 = 0x11;
    pub const BRANCH: u8 = 0x12;
    pub const BRANCH_IF_TRUE: u8 = 0x13;
    pub const BRANCH_IF_FALSE: u8 = 0x14;
    pub const CATCH_BLOCK: u8 = 0x15;
    pub const THROW: u8 = 0x16;
    pub const MAKE_ENV: u8 = 0x17;
    pub const RETURN: u8 = 0x18;
    pub const PUSH: u8 = 0x19;
    pub const POP: u8 = 0x1a;
    pub const CALL: u8 = 0x1b;
    pub const VIRT_CALL: u8 = 0x1c;
    pub const TAIL_CALL: u8 = 0x1d;
    pub const NEW: u8 = 0x1e;
    pub const GC: u8 = 0x1f;
    pub const GC_SAFEPOINT: u8 = 0x20;
    pub const ADD: u8 = 0x21;
    pub const SUB: u8 = 0x22;
    pub const DIV: u8 = 0x23;
    pub const MUL: u8 = 0x24;
    pub const MOD: u8 = 0x25;
    pub const RSH: u8 = 0x26;
    pub const LSH: u8 = 0x27;
    pub const GREATER: u8 = 0x28;
    pub const LESS: u8 = 0x29;
    pub const LESS_OR_EQUAL: u8 = 0x2a;
    pub const GREATER_OR_EQUAL: u8 = 0x2b;
    pub const EQUAL: u8 = 0x2c;
    pub const NOT_EQUAL: u8 = 0x2d;
    pub const AND: u8 = 0x2e;
    pub const OR: u8 = 0x2f;
    pub const XOR: u8 = 0x30;
    pub const NOT: u8 = 0x31;
    pub const NEG: u8 = 0x32;
    pub const MOVE: u8 = 0x33;
}
