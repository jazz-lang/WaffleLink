use crate::prelude::*;

pub enum Op {
    Add(u8, u8, u8, u32),
    Sub(u8, u8, u8, u32),
    Mul(u8, u8, u8, u32),
    Div(u8, u8, u8, u32),
    Mod(u8, u8, u8, u32),
    Lt(u8, u8, u8, u32),
    Le(u8, u8, u8, u32),
    Eq(u8, u8, u8, u32),
    Ne(u8, u8, u8, u32),
    Gt(u8, u8, u8, u32),
    Ge(u8, u8, u8, u32),
    And(u8, u8, u8, u32),
    Or(u8, u8, u8, u32),
    Xor(u8, u8, u8, u32),
    Shl(u8, u8, u8, u32),
    Shr(u8, u8, u8, u32),
    Connect(u8, u8, u8),
    Neg(u8, u8, u32),
    Flip(u8, u8, u32),
    LdNil(u8),
    LdBool(u8, u8, u8),
    LdInt(u8, i32),
    LdConst(u8, u32),
    Move(u8, u8),
    GetGbl(u8, u32),
    SetGbl(u8, u32),
    GetUpv(u8, u32),
    SetUpv(u8, u32),
    Jmp(i32),
    JmpT(u8, i32),
    JmpF(u8, i32),
    Call(u8, u8),
    Ret(u8, u8),
    Closure(u8, u32),
    GetMbr(u8, u8, u8),
    GetMet(u8, u8, u8),
    SetMbr(u8, u8, u8),
    GetIdx(u8, u8, i32),
    SetIdx(u8, u8, i32),
    SetSuper(u8, u8),
    Close(u8),
    Import(u8, u8, u8),
    Exblk(u8, u32),
    Catch(u8, u8, u8),
    Raise(u8, u8, u8),
    Class(u32),
}

#[repr(C)]
pub union Instruction {
    pub label: u32,
    pub u32: [u32; 2],
    pub i32: [i32; 2],
    pub i16: [i16; 4],
    pub u16: [u16; 4],
    pub ssw: ([i16; 2], u32),
    pub jump: ([i16; 2], i32),
    pub u64: u64,
    pub class: Handle<Class>,
    pub slot: *mut PolyICSlot,
}

pub struct PolyICSlot {
    pub class: Handle<Class>,
    pub ix: u32,
}

impl GcObject for Instruction {
    fn visit_references(&self, tracer: &mut Tracer<'_>) {
        unsafe {
            self.class.visit_references(tracer);
        }
    }
}

#[rustfmt::skip]
#[repr(u8)]
#[derive(Copy, Clone,Ord, PartialOrd, Eq, PartialEq)]
pub enum Opcode {
    ADD,      /*  A, B, C  |   R(A) <- RK(B) + RK(C) */
    SUB,      /*  A, B, C  |   R(A) <- RK(B) - RK(C) */
    MUL,      /*  A, B, C  |   R(A) <- RK(B) * RK(C) */
    DIV,      /*  A, B, C  |   R(A) <- RK(B) / RK(C) */
    MOD,      /*  A, B, C  |   R(A) <- RK(B) % RK(C) */
    LT,       /*  A, B, C  |   R(A) <- RK(B) < RK(C) */
    LE,       /*  A, B, C  |   R(A) <- RK(B) <= RK(C) */
    EQ,       /*  A, B, C  |   R(A) <- RK(B) == RK(C) */
    NE,       /*  A, B, C  |   R(A) <- RK(B) != RK(C) */
    GT,       /*  A, B, C  |   R(A) <- RK(B) > RK(C) */
    GE,       /*  A, B, C  |   R(A) <- RK(B) >= RK(C) */
    AND,      /*  A, B, C  |   R(A) <- RK(B) & RK(C) */
    OR,       /*  A, B, C  |   R(A) <- RK(B) | RK(C) */
    XOR,      /*  A, B, C  |   R(A) <- RK(B) ^ RK(C) */
    SHL,      /*  A, B, C  |   R(A) <- RK(B) << RK(C) */
    SHR,      /*  A, B, C  |   R(A) <- RK(B) >> RK(C) */
    CONNECT,  /*  A, B, C  |   R(A) <- connect(RK(B, RK(C)) */
    NEG,      /*  A, B     |   R(A) <- -RK(B) */
    FLIP,     /*  A, B     |   R(A) <- ~RK(B) */
    LDNIL,    /*  A        |   R(A) <- nil */
    LDBOOL,   /*  A, B, C  |   R(A) <- cast_bool(B, if(C): pc++ */
    LDINT,    /*  A, sBx   |   R(A) <- sBx */
    LDCONST,  /*  A, Bx    |   R(A) <- K(Bx) */
    MOVE,     /*  A, B, C  |   R(A) <- RK(B) */
    GETGBL,   /*  A, Bx    |   R(A) <- GLOBAL(Bx) */
    SETGBL,   /*  A, Bx    |   R(A) -> GLOBAL(Bx) */
    GETUPV,   /*  A, Bx    |   R(A) <- UPVALUE(Bx)*/
    SETUPV,   /*  A, Bx    |   R(A) -> UPVALUE(Bx)*/
    JMP,      /*  sBx      |   pc <- pc + sBx */
    JMPT,     /*  A, sBx   |   if(R(A)): pc <- pc + sBx  */
    JMPF,     /*  A, sBx   |   if(not R(A)): pc <- pc + sBx  */
    CALL,     /*  A, B     |   CALL(R(A, B) */
    RET,      /*  A, B     |   if (R(A)) R(-1) <- RK(B) else R(-1) <- nil */
    CLOSURE,  /*  A, Bx    |   R(A) <- CLOSURE(proto_table[Bx])*/
    GETMBR,   /*  A, B, C  |   R(A) <- RK(B).RK(C) */
    GETMET,   /*  A, B, C  |   R(A) <- RK(B).RK(C, R(A+1) <- RK(B) */
    SETMBR,   /*  A, B, C  |   R(A).RK(B) <- RK(C) */
    GETIDX,   /*  A, B, C  |   R(A) <- RK(B)[RK(C)] */
    SETIDX,   /*  A, B, C  |   R(A)[RK(B)] <- RK(C) */
    SETSUPER, /*  A, B     |   class:R(A) set super with class:RK(B) */
    CLOSE,    /*  A        |   close upvalues */
    IMPORT,   /*  A, B, C  |   IF (A == C) import module name as RK(B) to RK(A, ELSE from module RK(C) import name as RK(B) to RK(A) */
    EXBLK,    /*  A, Bx    |   ... */
    CATCH,    /*  A, B, C  |   ... */
    RAISE,    /*  A, B, C  |   ... */
    CLASS,    /*  Bx       |   init class in K[Bx] */
}

const_assert!(core::mem::size_of::<Opcode>() == 1);

pub const IOP_BITS: u32 = 6;
pub const IRA_BITS: u32 = 8;
pub const IRKB_BITS: u32 = 9;
pub const IRKC_BITS: u32 = 9;

pub const IRKC_POS: u32 = 0;
pub const IRKB_POS: u32 = IRKC_POS + IRKC_BITS;
pub const IRA_POS: u32 = IRKB_POS + IRKB_BITS;
pub const IOP_POS: u32 = IRA_POS + IRA_BITS;
pub const IAX_BITS: u32 = IRA_BITS + IRKB_BITS + IRKC_BITS;
pub const IBX_BITS: u32 = IRKC_BITS + IRKB_BITS;

pub const fn ins_mask(pos: u32, bits: u32) -> u32 {
    ((1 << (bits)) - 1) << (pos)
}

pub const fn ins_getx(i: u32, mask: u32, pos: u32) -> u32 {
    ((i) & (mask)) >> (pos)
}

pub const fn ins_setx(v: u32, mask: u32, pos: u32) -> u32 {
    ((v) << (pos)) & (mask)
}

pub const fn is_k(v: u32) -> bool {
    ((v) & (1 << (IRKB_BITS - 1))) != 0
}

pub const fn set_k(v: u32) -> u32 {
    (v) | (1 << (IRKB_BITS - 1))
}
pub const fn kr2idx(v: u32) -> u32 {
    v & 0xff
}

pub const fn is_kb(v: u32) -> bool {
    ((v) & (1 << (IRA_POS - 1))) != 0
}

pub const fn is_kc(v: u32) -> bool {
    ((v) & (1 << (IRKB_POS - 1))) != 0
}

pub const IOP_MASK: u32 = ins_mask(IOP_POS, IOP_BITS);
pub const IRA_MASK: u32 = ins_mask(IRA_POS, IRA_BITS);
pub const IRKB_MASK: u32 = ins_mask(IRKB_POS, IRKB_BITS);
pub const IRKC_MASK: u32 = ins_mask(IRKC_POS, IRKC_BITS);
pub const IAX_MASK: u32 = ins_mask(0, IAX_BITS);
pub const IBX_MASK: u32 = ins_mask(0, IBX_BITS);
pub const ISBX_MAX: i32 = (IBX_MASK >> 1) as i32;
pub const ISBX_MIN: i32 = -((ISBX_MAX as u32 - 1) as i32);

pub const fn iget_op(i: u32) -> u8 {
    ins_getx(i, IOP_MASK, IOP_POS) as _
}

pub const fn iget_ra(i: u32) -> u32 {
    ins_getx(i, IRA_MASK, IRA_POS)
}

pub const fn iget_rkb(i: u32) -> u32 {
    ins_getx(i, IRKB_MASK, IRKB_POS)
}
pub const fn iget_rkc(i: u32) -> u32 {
    ins_getx(i, IRKC_MASK, IRKC_POS)
}

pub const fn iget_bx(i: u32) -> u32 {
    ins_getx(i, IBX_MASK, 0)
}

pub const fn iget_sbx(i: u32) -> i32 {
    iget_bx(i) as i32 - ISBX_MAX
}
pub const fn iset_op(i: u32) -> u32 {
    ins_setx(i, IOP_MASK, IOP_POS)
}

pub const fn iset_ra(i: u32) -> u32 {
    ins_setx(i, IRA_MASK, IRA_POS)
}

pub const fn iset_rkb(i: u32) -> u32 {
    ins_setx(i, IRKB_MASK, IRKB_POS)
}
pub const fn iset_rkc(i: u32) -> u32 {
    ins_setx(i, IRKC_MASK, IRKC_POS)
}

pub const fn iset_bx(i: u32) -> u32 {
    ins_setx(i, IBX_MASK, 0)
}

pub const fn iset_sbx(i: u32) -> u32 {
    iset_bx((i as i32 + ISBX_MAX) as u32)
}
