/*
*   Copyright (c) 2020 Adel Prokurov
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash)]
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
    LoadUpvalue(u16, u16),
    StoreUpvalue(u16, u16),
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
    ToBoolean(u16, u16),
}

impl Instruction {
    pub fn branch_targets(&self) -> Vec<usize> {
        match &self {
            Instruction::Branch(x)
            | Instruction::BranchIfFalse(_, x)
            | Instruction::BranchIfTrue(_, x) => vec![*x as _],
            Instruction::ConditionalBranch(_, x, y) => vec![*x as _, *y as _],
            _ => vec![],
        }
    }

    pub fn get_defs(&self) -> Vec<usize> {
        let mut def_set = std::collections::HashSet::new();
        macro_rules! vreg {
            ($e: expr) => {
                $e
            };
        }
        match self {
            Instruction::LoadStaticById(r, _) => {
                def_set.insert(vreg!(*r));
            }
            Instruction::Move(to, _from) => {
                def_set.insert(vreg!(*to));
                //(vreg!(*from));
            }
            Instruction::LoadUpvalue(r, _) => {
                def_set.insert(vreg!(*r));
            }
            Instruction::StoreUpvalue(r, _) => {
                //(vreg!(*r));
            }
            Instruction::LoadById(x, y, _) | Instruction::LoadByIndex(x, y, _) => {
                def_set.insert(vreg!(*x));
                //(vreg!(*y));
            }
            Instruction::LoadByValue(x, y, z) => {
                def_set.insert(vreg!(*x));
                //(vreg!(*y));
                //(vreg!(*z));
            }
            Instruction::StoreById(x, y, _) | Instruction::StoreByIndex(x, y, _) => {
                //modified_set.insert(vreg!(*x));
                //(vreg!(*y));
            }
            Instruction::StoreByValue(x, y, z) => {
                //modified_set.insert(vreg!(*x));
                //(vreg!(*y));
                //(vreg!(*z));
            }
            Instruction::LoadNumber(r, _) => {
                def_set.insert(vreg!(*r));
            }
            Instruction::LoadInt(r, _) => {
                def_set.insert(vreg!(*r));
            }
            Instruction::LoadConst(r, _) => {
                def_set.insert(vreg!(*r));
            }
            Instruction::LoadUndefined(r) => {
                def_set.insert(vreg!(*r));
            }
            Instruction::LoadNull(r) => {
                def_set.insert(vreg!(*r));
            }
            Instruction::CatchBlock(r, _) => {
                def_set.insert(vreg!(*r));
            }
            Instruction::Throw(r) => {}
            Instruction::VirtCall(r0, r1, r2, _) => {
                def_set.insert(vreg!(*r0));
                //(vreg!(*r1));
                //(vreg!(*r2));
            }
            Instruction::Call(r0, r1, _) | Instruction::TailCall(r0, r1, _) => {
                def_set.insert(vreg!(*r0));
                //(vreg!(*r1));
            }
            Instruction::New(r0, r1, _) => {
                def_set.insert(vreg!(*r0));
                //(vreg!(*r1));
            }
            Instruction::Return(Some(r)) => {
                //(vreg!(*r));
            }
            Instruction::Return(None) => (),
            Instruction::MakeEnv(r0, _) => {
                def_set.insert(vreg!(*r0));
                //modified_set.insert(vreg!(*r0));
            }
            Instruction::Push(r0) => {
                //(vreg!(*r0));
            }
            Instruction::Pop(r0) => {
                def_set.insert(vreg!(*r0));
            }

            Instruction::StoreStack(r, _) => {
                //(vreg!(*r));
            }
            Instruction::LoadStack(r, _) => {
                def_set.insert(vreg!(*r));
            }
            Instruction::Binary(_, r0, r1, r2) => {
                def_set.insert(vreg!(*r0));
                //(vreg!(*r1));
                //(vreg!(*r2));
            }
            Instruction::LoadTrue(r) | Instruction::LoadFalse(r) => {
                def_set.insert(vreg!(*r));
            }
            Instruction::Unary(_, r0, r1) => {
                def_set.insert(vreg!(*r0));
                //(vreg!(*r1));
            }
            Instruction::BranchIfFalse(r0, _)
            | Instruction::BranchIfTrue(r0, _)
            | Instruction::ConditionalBranch(r0, _, _) => {
                //(vreg!(*r0));
            }
            Instruction::ToBoolean(r0, _) => {
                def_set.insert(vreg!(*r0));
            }
            Instruction::LoadThis(r0) => {
                def_set.insert(vreg!(*r0));
            }
            Instruction::SetThis(r0) => {
                //(vreg!(*r0));
            }
            Instruction::LoadCurrentModule(r0) => {
                def_set.insert(vreg!(*r0));
            }
            _ => {}
        };
        def_set.iter().map(|x| *x as usize).collect()
    }
    pub fn get_uses(&self) -> Vec<usize> {
        let mut use_set = std::collections::HashSet::new();
        macro_rules! vreg {
            ($e: expr) => {
                $e
            };
        }
        match self {
            Instruction::Move(to, from) => {
                use_set.insert(vreg!(*from));
            }
            Instruction::LoadUpvalue(r, _) => {}
            Instruction::StoreUpvalue(r, _) => {
                use_set.insert(vreg!(*r));
            }
            Instruction::LoadById(x, y, _) | Instruction::LoadByIndex(x, y, _) => {
                use_set.insert(vreg!(*y));
            }
            Instruction::LoadByValue(x, y, z) => {
                use_set.insert(vreg!(*y));
                use_set.insert(vreg!(*z));
            }
            Instruction::StoreById(x, y, _) | Instruction::StoreByIndex(x, y, _) => {
                use_set.insert(vreg!(*y));
                use_set.insert(vreg!(*y));
            }
            Instruction::StoreByValue(x, y, z) => {
                use_set.insert(vreg!(*x));
                use_set.insert(vreg!(*y));
                use_set.insert(vreg!(*z));
            }
            Instruction::LoadNumber(r, _) => {}
            Instruction::LoadInt(r, _) => {}
            Instruction::LoadConst(r, _) => {}
            Instruction::LoadUndefined(r) => {}
            Instruction::CatchBlock(r, _) => {}
            Instruction::Throw(r) => {
                use_set.insert(vreg!(*r));
            }
            Instruction::VirtCall(r0, r1, r2, _) => {
                use_set.insert(vreg!(*r1));
                use_set.insert(vreg!(*r2));
            }
            Instruction::Call(r0, r1, _) | Instruction::TailCall(r0, r1, _) => {
                use_set.insert(vreg!(*r1));
            }
            Instruction::New(r0, r1, _) => {
                use_set.insert(vreg!(*r1));
            }
            Instruction::Return(Some(r)) => {
                use_set.insert(vreg!(*r));
            }
            Instruction::Return(None) => (),
            Instruction::MakeEnv(r0, _) => {
                use_set.insert(vreg!(*r0));
            }
            Instruction::Push(r0) => {
                use_set.insert(vreg!(*r0));
            }
            Instruction::Pop(r0) => {}

            Instruction::StoreStack(r, _) => {
                use_set.insert(vreg!(*r));
            }
            Instruction::LoadStack(r, _) => {}
            Instruction::Binary(_, _r0, r1, r2) => {
                use_set.insert(vreg!(*r1));
                use_set.insert(vreg!(*r2));
            }
            Instruction::Unary(_, r0, r1) => {
                use_set.insert(vreg!(*r1));
            }
            Instruction::BranchIfFalse(r0, _)
            | Instruction::BranchIfTrue(r0, _)
            | Instruction::ConditionalBranch(r0, _, _) => {
                use_set.insert(vreg!(*r0));
            }
            Instruction::LoadThis(r0) => {}
            Instruction::SetThis(r0) => {
                use_set.insert(vreg!(*r0));
            }
            Instruction::ToBoolean(_, r1) => {
                use_set.insert(vreg!(*r1));
            }
            Instruction::LoadCurrentModule(r0) => {}
            _ => {}
        }

        use_set.iter().map(|x| *x as usize).collect()
    }

    pub fn replace_reg(&mut self, temp: usize, to: usize) {
        macro_rules! r {
            ($t: expr) => {{
                if *$t == temp as u16 {
                    *$t = to as u16;
                }
            }};
            ($($t: expr),*) => {
                {$(
                    r!($t);
                )*
                }
            };
        }
        match self {
            Instruction::LoadNull(r)
            | Instruction::LoadUndefined(r)
            | Instruction::LoadInt(r, _)
            | Instruction::LoadNumber(r, _)
            | Instruction::LoadTrue(r)
            | Instruction::LoadUpvalue(r, _)
            | Instruction::StoreUpvalue(r, _)
            | Instruction::StoreStack(r, _)
            | Instruction::LoadStack(r, _)
            | Instruction::LoadStaticById(r, _)
            | Instruction::StoreStaticById(r, _)
            | Instruction::ConditionalBranch(r, _, _)
            | Instruction::BranchIfFalse(r, _)
            | Instruction::BranchIfTrue(r, _)
            | Instruction::CatchBlock(r, _)
            | Instruction::Throw(r)
            | Instruction::MakeEnv(r, _)
            | Instruction::Return(Some(r))
            | Instruction::Pop(r)
            | Instruction::Push(r)
            | Instruction::LoadConst(r, _)
            | Instruction::LoadThis(r)
            | Instruction::SetThis(r)
            | Instruction::LoadCurrentModule(r)
            | Instruction::LoadFalse(r) => r!(r),
            Instruction::LoadById(r1, r2, _)
            | Instruction::Move(r1, r2)
            | Instruction::Unary(_, r1, r2)
            | Instruction::Call(r1, r2, _)
            | Instruction::TailCall(r1, r2, _)
            | Instruction::StoreById(r1, r2, _)
            | Instruction::ToBoolean(r1, r2)
            | Instruction::New(r1, r2, _)
            | Instruction::LoadStaticByValue(r1, r2) => r!(r1, r2),
            Instruction::LoadByValue(r1, r2, r3)
            | Instruction::StoreByValue(r1, r2, r3)
            | Instruction::VirtCall(r1, r2, r3, _)
            | Instruction::Binary(_, r1, r2, r3) => r!(r1, r2, r3),
            _ => (),
        }
    }

    pub fn args(&self) -> Vec<u64> {
        match *self {
            Instruction::LoadNull(r)
            | Instruction::LoadUndefined(r)
            | Instruction::LoadTrue(r)
            | Instruction::LoadFalse(r)
            | Instruction::LoadThis(r)
            | Instruction::SetThis(r)
            | Instruction::Push(r)
            | Instruction::Pop(r)
            | Instruction::Throw(r)
            | Instruction::LoadCurrentModule(r) => vec![r as u64],
            Instruction::Binary(op, r1, r2, r3) => {
                vec![op as u8 as u64, r1 as u64, r2 as u64, r3 as u64]
            }
            Instruction::VirtCall(r1, r2, r3, argc) => vec![r1 as u64, r2 as u64, r3 as u64],
            Instruction::TailCall(r1, r2, argc)
            | Instruction::Call(r1, r2, argc)
            | Instruction::New(r1, r2, argc) => vec![r1 as u64, r2 as u64, argc as u64],
            Instruction::Unary(op, r1, r2) => vec![op as u64, r1 as u64, r2 as u64],
            Instruction::LoadConst(r1, c1) => vec![r1 as u64, c1 as u64],
            Instruction::LoadStack(r1, s1) => vec![r1 as u64, s1 as u64],
            Instruction::StoreStack(r1, s1) => vec![r1 as u64, s1 as u64],
            Instruction::Branch(target) => vec![target as u64],
            Instruction::BranchIfFalse(r1, target) | Instruction::BranchIfTrue(r1, target) => {
                vec![r1 as u64, target as u64]
            }
            Instruction::ConditionalBranch(r1, t1, t2) => vec![r1 as u64, t1 as u64, t2 as u64],
            Instruction::CatchBlock(r1, t1) => vec![r1 as u64, t1 as u64],
            Instruction::LoadByValue(r1, r2, r3) | Instruction::StoreByValue(r1, r2, r3) => {
                vec![r1 as u64, r2 as u64, r3 as u64]
            }
            Instruction::StoreById(r1, r2, r3) | Instruction::LoadById(r1, r2, r3) => {
                vec![r1 as u64, r2 as u64, r3 as u64]
            }
            Instruction::StoreStaticByValue(r1, r2) | Instruction::LoadStaticByValue(r1, r2) => {
                vec![r1 as u64, r2 as u64]
            }
            Instruction::StoreStaticById(r1, r2) | Instruction::LoadStaticById(r1, r2) => {
                vec![r1 as u64, r2 as u64]
            }

            Instruction::LoadUpvalue(r1, r2) | Instruction::StoreUpvalue(r1, r2) => {
                vec![r1 as u64, r2 as u64]
            }
            Instruction::LoadInt(r1, n) => vec![r1 as u64, n as u64],
            Instruction::LoadNumber(r1, n) => vec![r1 as u64, n as u64],
            Instruction::MakeEnv(r1, c) => vec![r1 as u64, c as u64],
            Instruction::Return(Some(r)) => vec![r as u64],
            Instruction::Move(r1, r2) => vec![r1 as u64, r2 as u64],
            Instruction::LoadByIndex(r1, r2, i) | Instruction::StoreByIndex(r1, r2, i) => {
                vec![r1 as u64, r2 as u64, i as u64]
            }
            Instruction::ToBoolean(r1, r2) => vec![r1 as u64, r2 as u64],
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
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
    pub const LOAD_UPVALUE: u8 = 0x38;
    pub const STORE_UPVALUE: u8 = 0x39;
    pub const TO_BOOLEAN: u8 = 0x3a;
}
