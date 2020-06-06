#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Op {
    AccNull,
    AccConst,
    AccTrue,
    AccFalse,
    AccThis,
    AccInt,
    AccStack,
    AccGlobal,
    AccEnv,
    AccField,
    AccArray,
    AccIndex,
    SetStack,
    SetGlobal,
    SetEnv,
    SetField,
    SetArray,
    SetIndex,
    SetThis,
    Push,
    Pop,
    Call,
    ObjCall,
    Jump,
    JumpIf,
    JumpIfNot,
    PushCatch,
    Throw,
    PopCatch,
    Ret,
    MakeEnv,
    MakeArray,
    Bool,
    IsNull,
    IsNotNull,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    UShr,
    Or,
    Xor,
    And,
    Eq,
    Neq,
    Gt,
    Ge,
    Lt,
    Le,
    Not,
    TypeOf,
    Compare,
    Hash,
    New,
    JumpTable,
    Apply,
    AccStack0,
    AccStack1,
    AccIndex0,
    AccIndex1,
    PhysCompare,
    TailCall,
    Loop,
    MakeArray2,
    AccInt32,
    Last,
}

use super::arc::ArcWithoutWeak;
use super::state::*;
use super::threads::*;
use super::value::*;
#[derive(Copy, Clone)]
pub union InsEnc {
    pub fun: fn(state: &mut super::interp::Interpreter, pc: Pc),
    pub i8: i8,
    pub i16: i16,
    pub i32: i32,
    pub jump: [i16; 2],
}

#[derive(Copy, Clone)]
pub struct Ins {
    enc: InsEnc,
}
impl Ins {
    pub fn func(self) -> fn(state: &mut super::interp::Interpreter, pc: Pc) {
        unsafe { self.enc.fun }
    }

    pub fn i8(self) -> i8 {
        unsafe { self.enc.i8 }
    }

    pub fn i16(self) -> i16 {
        unsafe { self.enc.i16 }
    }
    pub fn i32(self) -> i32 {
        unsafe { self.enc.i32 }
    }

    pub fn jump(self) -> [i16; 2] {
        unsafe { self.enc.jump }
    }
}

use super::gc::Collectable;
impl Collectable for Ins {
    fn walk_references(&self, _trace: &mut dyn FnMut(*const crate::gc::Handle<dyn Collectable>)) {
        ()
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Pc {
    ins: *mut Ins,
}

impl Pc {
    pub fn new(code: &[Ins]) -> Self {
        Self {
            ins: code.as_ptr() as *mut Ins,
        }
    }
    #[optimize(speed)]
    pub fn advance(&mut self) -> Ins {
        unsafe {
            let c = *self.ins;
            self.ins = self.ins.offset(1);
            c
        }
    }

    pub fn current(&self) -> Ins {
        unsafe { *self.ins }
    }
}
