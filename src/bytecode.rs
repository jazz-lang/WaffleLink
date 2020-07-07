use crate::value::Value;
#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum Ins {
    Enter,
    Move(u8, u8),
    Swap(u8, u8),
    MoveInt(i32, u8),
    LoadArg(u32, u8),
    SetArg(u8, u32),
    Load(u8, u8, u8),
    Store(u8, u8, u8),
    LoadId(u32, u8, u8),
    StoreId(u8, u32, u8),
    LoadConst(u32, u8),

    LoadGlobal(u32 /* id constant */, u8),
    StoreGlobal(u8, u32 /* id constant */),

    Add(u8, u8, u8),
    Sub(u8, u8, u8),
    Mul(u8, u8, u8),
    Div(u8, u8, u8),
    Rem(u8, u8, u8),
    LShift(u8, u8, u8),
    RShift(u8, u8, u8),
    URShift(u8, u8, u8),
    Equal(u8, u8, u8),
    NotEqual(u8, u8, u8),
    Greater(u8, u8, u8),
    GreaterOrEqual(u8, u8, u8),
    Less(u8, u8, u8),
    LessOrEqul(u8, u8, u8),
    Safepoint,
    LoopHint,
    Jump(i32),
    JumpIfZero(u8, i32),
    JumpIfNotZero(u8, i32),
    TryCatch(u32 /* try block */, u32 /* catch block */),
    GetException(u8),
    Call(
        u8,  /* this */
        u8,  /* function */
        u8,  /* dest */
        u32, /* argc */
    ),
    TailCall(
        u8,  /* this */
        u8,  /* function */
        u32, /* argc */
    ),
    New(
        u8,  /* constructor or object */
        u8,  /* dest */
        u32, /* argc */
    ),

    Return(u8),
}

pub struct CodeBlock {
    pub num_vars: u32,
    pub num_args: u32,
    pub instructions: Vec<Ins>,
    pub constants: Vec<Value>,
}

impl CodeBlock {
    pub fn new() -> Self {
        Self {
            num_vars: 0,
            instructions: vec![],
            num_args: 0,
            constants: vec![],
        }
    }
}
