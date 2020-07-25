#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Opcode {
    IConst,
    FConst,
    Move,
    Load,
    Store,
    Call(CallConv, u32),
    AddOvfUI,
    SubOvfUI,
    MulOVfUI,
    AddOvfI,
    SubOvfI,
    MulOvfI,
    AddUI,
    AddI,
    SubUI,
    SubI,
    DivUI,
    DivI,
    MulUI,
    MulI,
    ModUI,
    ModI,
    RemI,
    RemF,
    RemD,
    DivF,
    DivD,
    ModF,
    ModD,
    OrI,
    OrUI,
    AndI,
    AndUI,
    XorI,
    XorUI,
    LShiftUI,
    LShiftI,
    RShiftUI,
    RShiftI,

    /* value operations */
    ValueAdd,
    ValueSub,
    ValueDiv,
    ValueMul,
    ValueMod,
    ValueRem,
    ValueLShift,
    ValueRShift,
    ValueURShift,
    ValueBitAnd,
    ValueBitOr,
    ValueBitXor,
    ValueCompare(Condition),

    Compare(Condition),

    /// Guards used in optimizing and tracing jit, all guards have index to baseline JIT code map.
    GuardInt32(u32),
    GuardAnyNum(u32),
    GuardNum(u32),
    GuardArray(u32),
    GuardString(u32),
    GuardObject(u32),
    GuardZero(u32),
    GuardNonZero(u32),
    GuardType(u32, super::Type),
    /// Guard fails if condition is true
    GuardCmp(Condition, u32),
    /// Guard fails if condition is false
    GuardNCmp(Condition, u32),
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Condition {
    UnsignedLess,
    UnsignedGreater,
    UnsignedLessOrEqual,
    UnsignedGreaterOrEqual,
    Equal,
    NotEqual,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum CallConv {
    SystemV,
    Win64,
    FastCall,
}

/// Instruction executed at end of all basic blocks
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Terminator {
    Branch(u32),
    ConditionalBranch(Condition, u32, u32),
    TailCall(CallConv, u32),
}
