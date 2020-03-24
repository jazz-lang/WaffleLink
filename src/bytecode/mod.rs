macro_rules! opcodes {
    ($m: ident) => {
        include!("opcodes.def");
    };
}

macro_rules! declare {
    ($op: ident,$x: expr,$_xx: expr,$_x2: literal) => {
        #[doc($_x2)]
        pub const $op: u32 = $x;
    };
    ($op: ident,$x: expr,$_xx: expr) => {
        pub const $op: u32 = $x;
    };
}

opcodes!(declare);

pub mod basicblock;

#[derive(Copy, Clone, Debug)]
pub enum Opcode {
    LoadConst(u8, u32),
    LoadInt(u8, i32),
    LoadUpvalue(u8, u16),
    LoadById(u8, u8, u32),
    LoadByIndex(u8, u8, i32),
    LoadStaticById(u8, u32),
    StoreUpvalue(u8, u16),
    StoreById(u8, u8, u32),
    StoreByIndex(u8, u8, u32),
    StoreStaticById(u8, u32),
    LoadTrue(u8),
    LoadFalse(u8),
    LoadNil(u8),
    LoadUndef(u8),
    LoadStack(u8, u16),
    StoreStack(u8, u16),
    Push(u8),
    Pop(u8),
    Conditional(u8, u16, u16),
    Goto(u16),
    MakeEnv(u8, u8, u16),
    Ret(u8),
    Call(u8, u8, u8),
    VirtCall(u8, u8, u8, u8),
    TailCall(u8, u8, u8),
    New(u8, u8, u8),
    Safepoint,
    Add(u8, u8, u8),
    Sub(u8, u8, u8),
    Div(u8, u8, u8),
    Mul(u8, u8, u8),
    Mod(u8, u8, u8),
    Shl(u8, u8, u8),
    Shr(u8, u8, u8),
    UShr(u8, u8, u8),
    Eq(u8, u8, u8),
    Gt(u8, u8, u8),
    Lt(u8, u8, u8),
    Le(u8, u8, u8),
    Ge(u8, u8, u8),
    Neq(u8, u8, u8),
    Not(u8, u8, u8),
    UnaryPlus(u8, u8),
    LoadThis(u8),
    StoreThis(u8),
    Yield(u8),
    Popcnt(u16),
    LoadByValue(u8, u8, u8),
    StoreByValue(u8, u8, u8),
}
