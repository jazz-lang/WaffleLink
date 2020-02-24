extern crate regalloc as ra;
use ra::*;
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
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
    /// Branch B(B) if R(A) is true, otherwise Branch to B(C)
    ConditionalBranch(u16, u16, u16),
    /// Branch B(B)
    Branch(u16),
    /// Branch B(B) if R(A) is true
    BranchIfTrue(u16, u16),
    /// Branch B if R(a) is false
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
    LoadConst(u16, u32),
    LoadThis(u16),
    SetThis(u16),
    LoadCurrentModule(u16),
}

macro_rules! vreg {
    ($v: expr) => {
        if $v > 32 {
            Reg::new_virtual(RegClass::I64, $v as u32)
        } else {
            Reg::new_real(RegClass::I64, 1, $v as _)
        }
    };
}

impl Instruction {
    pub fn get_targets(&self) -> Vec<BlockIx> {
        match &self {
            Instruction::Branch(x)
            | Instruction::BranchIfFalse(_, x)
            | Instruction::BranchIfTrue(_, x) => vec![BlockIx::new(*x as u32)],
            Instruction::ConditionalBranch(_, x, y) => {
                vec![BlockIx::new(*x as u32), BlockIx::new(*y as u32)]
            }
            _ => vec![],
        }
    }

    pub fn get_reg_usage(&self) -> (Set<Reg>, Set<Reg>, Set<Reg>) {
        let mut def = Set::<Reg>::empty();
        let mut m0d = Set::<Reg>::empty();
        let mut uce = Set::<Reg>::empty();
        macro_rules! vreg {
            ($v: expr) => {
                if $v > 32 {
                    Reg::new_virtual(RegClass::I64, $v as u32)
                } else {
                    Reg::new_real(RegClass::I64, 1, $v as _)
                }
            };
        }
        match self {
            Instruction::Move(to, from) => {
                def.insert(vreg!(*to));
                uce.insert(vreg!(*from));
            }
            Instruction::LoadById(x, y, _) | Instruction::LoadByIndex(x, y, _) => {
                def.insert(vreg!(*x));
                uce.insert(vreg!(*y));
            }
            Instruction::LoadByValue(x, y, z) => {
                def.insert(vreg!(*x));
                uce.insert(vreg!(*y));
                uce.insert(vreg!(*z));
            }
            Instruction::StoreById(x, y, _) | Instruction::StoreByIndex(x, y, _) => {
                m0d.insert(vreg!(*x));
                uce.insert(vreg!(*y));
            }
            Instruction::StoreByValue(x, y, z) => {
                m0d.insert(vreg!(*x));
                uce.insert(vreg!(*y));
                uce.insert(vreg!(*z));
            }
            Instruction::LoadNumber(r, _) => {
                def.insert(vreg!(*r));
            }
            Instruction::LoadInt(r, _) => {
                def.insert(vreg!(*r));
            }
            Instruction::LoadConst(r, _) => {
                def.insert(vreg!(*r));
            }
            Instruction::LoadUndefined(r) => def.insert(vreg!(*r)),
            Instruction::LoadNull(r) => def.insert(vreg!(*r)),
            Instruction::CatchBlock(r, _) => def.insert(vreg!(*r)),
            Instruction::Throw(r) => uce.insert(vreg!(*r)),
            Instruction::VirtCall(r0, r1, r2, _) => {
                def.insert(vreg!(*r0));
                uce.insert(vreg!(*r1));
                uce.insert(vreg!(*r2));
            }
            Instruction::Call(r0, r1, _) | Instruction::TailCall(r0, r1, _) => {
                def.insert(vreg!(*r0));
                uce.insert(vreg!(*r1));
            }
            Instruction::New(r0, r1, _) => {
                def.insert(vreg!(*r0));
                uce.insert(vreg!(*r1));
            }
            Instruction::Return(Some(r)) => {
                uce.insert(vreg!(*r));
            }
            Instruction::Return(None) => (),
            Instruction::MakeEnv(r0, _) => {
                m0d.insert(vreg!(*r0));
            }
            Instruction::Push(r0) => {
                uce.insert(vreg!(*r0));
            }
            Instruction::Pop(r0) => {
                def.insert(vreg!(*r0));
            }

            Instruction::StoreStack(r, _) => {
                uce.insert(vreg!(*r));
            }
            Instruction::LoadStack(r, _) => {
                def.insert(vreg!(*r));
            }
            Instruction::Binary(_, r0, r1, r2) => {
                def.insert(vreg!(*r0));
                uce.insert(vreg!(*r1));
                uce.insert(vreg!(*r2));
            }
            Instruction::Unary(_, r0, r1) => {
                def.insert(vreg!(*r0));
                uce.insert(vreg!(*r1));
            }
            Instruction::BranchIfFalse(r0, _)
            | Instruction::BranchIfTrue(r0, _)
            | Instruction::ConditionalBranch(r0, _, _) => {
                uce.insert(vreg!(*r0));
            }
            Instruction::LoadThis(r0) => {
                def.insert(vreg!(*r0));
            }
            Instruction::SetThis(r0) => {
                uce.insert(vreg!(*r0));
            }
            Instruction::LoadCurrentModule(r0) => {
                def.insert(vreg!(*r0));
            }
            _ => {}
        }

        (def, m0d, uce)
    }

    pub fn map_regs_d_u(
        &mut self,
        map_defs: &Map<VirtualReg, RealReg>,
        map_uses: &Map<VirtualReg, RealReg>,
    ) {
        macro_rules! map {
            (use $r: ident $($rest:tt)*) => {
                {
                    *$r = map_uses
                        .get(&vreg!(*$r).to_virtual_reg())
                        .unwrap()
                        .get_index() as u16;
                    map!($($rest)*);
                }
            };
            (def $r: ident $($rest:tt)*) => {
                {
                    *$r = map_defs
                        .get(&vreg!(*$r).to_virtual_reg())
                        .unwrap()
                        .get_index() as u16;
                    map!($($rest)*);
                }
            };
            () => {};
            ($($t: tt)*) => {
                map!($($t)*);
            }
        }
        match self {
            Instruction::LoadStaticById(r0, _) => map!(def r0),
            Instruction::LoadStaticByValue(r0, r1) => map!(def r0 use r1),
            Instruction::StoreStaticById(r0, _) => map!(use r0),
            Instruction::StoreStaticByValue(r0, _) => map!(use r0),
            Instruction::LoadById(r0, r1, _) => map!(def r0 use r1),
            Instruction::StoreById(r0, r1, _) => map!(use r0 use r1),
            Instruction::StoreByValue(r0, r1, r2) => map!(use r0 use r1 use r2),
            Instruction::LoadByValue(r0, r1, r2) => map!(def r0 use r1 use r2),
            Instruction::LoadByIndex(r0, r1, _) => map!(def r0 use r1),
            Instruction::StoreByIndex(r0, r1, _) => map!(use r0 use r1),
            Instruction::Throw(r) => map!(use r),
            Instruction::CatchBlock(r, _) => map!(def r),
            Instruction::SetThis(r) => map!(use r),
            Instruction::LoadThis(r) => map!(def r),
            Instruction::New(r0, r1, _) => map!(def r0 use r1),
            Instruction::LoadUndefined(r) | Instruction::LoadNull(r) => map!(def r),
            Instruction::LoadTrue(r0) | Instruction::LoadFalse(r0) => map!(def r0),
            Instruction::Binary(_, r0, r1, r2) => {
                map!(def r0 use r1 use r2);
            }

            Instruction::LoadNumber(r0, _) => {
                map!(def r0);
            }
            Instruction::LoadInt(r0, _) => {
                map!(def r0);
            }
            Instruction::LoadConst(r0, _) => {
                map!(def r0);
            }
            Instruction::Call(r0, r1, _) | Instruction::TailCall(r0, r1, _) => {
                map!(def r0 use r1);
            }
            Instruction::VirtCall(r0, r1, r2, _) => map!(def r0 use r1 use r2),
            Instruction::Return(r0) => {
                if let Some(r0) = r0 {
                    map!(use r0);
                }
            }
            Instruction::MakeEnv(r0, _) => {
                map!(use r0);
            }
            Instruction::Push(r0) => {
                map!(use r0);
            }
            Instruction::Pop(r0) => {
                map!(def r0);
            }
            Instruction::Move(r0, r1) => {
                map!(def r0 use r1);
            }
            Instruction::StoreStack(r0, _) => {
                map!(use r0);
            }
            Instruction::LoadStack(r0, _) => {
                map!(def r0);
            }
            Instruction::Unary(_, r0, r1) => map!(def r0 use r1),
            Instruction::BranchIfFalse(r0, _)
            | Instruction::BranchIfTrue(r0, _)
            | Instruction::ConditionalBranch(r0, _, _) => {
                map!(use r0);
            }

            Instruction::LoadCurrentModule(r0) => map!(def r0),

            _ => {}
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    pub const LOAD_CONST: u8 = 0x34;
    pub const LOAD_THIS: u8 = 0x35;
    pub const SET_THIS: u8 = 0x36;
    pub const LOAD_CURRENT_MODULE: u8 = 0x37;
}
