use super::virtual_reg::*;
use derive_more::Display;

#[derive(Copy, Clone, Display)]
pub enum Ins {
    // dst = src
    #[display(fmt = "mov {},{}", dst, src)]
    Mov {
        dst: VirtualRegister,
        src: VirtualRegister,
    },
    #[display(fmt = "movi {},{}", dst, imm)]
    // dst = imm
    LoadI32 { dst: VirtualRegister, imm: i32 },
    // dst = new Generator(src)
    #[display(fmt = "new_generator_function {},{}", dst, src)]
    NewGeneratorFunction {
        dst: VirtualRegister,
        src: VirtualRegister,
    },
    #[display(fmt = "close_env {}, {}, {}-{}", dst, function, begin, end)]
    // function.env = registers[begin..end]
    CloseEnv {
        dst: VirtualRegister,
        function: VirtualRegister,
        begin: VirtualRegister,
        end: VirtualRegister,
    },
    #[display(fmt = "close_env {},{}", dst, function)]
    // function.env = registers[begin..end]
    CloseEnvNoArgs {
        dst: VirtualRegister,
        function: VirtualRegister,
    },
    // dst = function.apply(this,registers[begin..end])
    #[display(fmt = "call {},{},{},{}-{}", dst, function, this, begin, end)]
    Call {
        dst: VirtualRegister,
        function: VirtualRegister,
        this: VirtualRegister,
        begin: VirtualRegister,
        end: VirtualRegister,
    },
    #[display(fmt = "call {},{},{}", dst, function, this)]
    CallNoArgs {
        dst: VirtualRegister,
        function: VirtualRegister,
        this: VirtualRegister,
    },
    #[display(fmt = "tailcall {},{},{},{}-{}", dst, function, this, begin, end)]
    TailCall {
        dst: VirtualRegister,
        function: VirtualRegister,
        this: VirtualRegister,
        begin: VirtualRegister,
        end: VirtualRegister,
    },
    // dst = yield res
    #[display(fmt = "yield {},{}", dst, res)]
    Yield {
        dst: VirtualRegister,
        res: VirtualRegister,
    },
    #[display(fmt = "return {}", val)]
    // return val
    Return { val: VirtualRegister },
    // dst = await on
    #[display(fmt = "await {},{}", dst, on)]
    Await {
        dst: VirtualRegister,
        on: VirtualRegister,
    },
    #[display(fmt = "try_catch [%{}] [%{}],->{}", try_, catch, reg)]
    TryCatch {
        // try block, we jump to it immediatly
        try_: u32,
        // catch block
        catch: u32,
        // register where to store exception if thrown.
        reg: VirtualRegister,
    },
    #[display(fmt = "throw {}", src)]
    Throw { src: VirtualRegister },
    #[display(fmt = "add {},{},{}", dst, lhs, src)]
    Add {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "sub {},{},{}", dst, lhs, src)]
    Sub {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "div {},{},{}", dst, lhs, src)]
    Div {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "mul {},{},{}", dst, lhs, src)]
    Mul {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "mod {},{},{}", dst, lhs, src)]
    Mod {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    /// Creates string from two values
    #[display(fmt = "concat {},{},{}", dst, lhs, src)]
    Concat {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
    },
    #[display(fmt = "shr {},{},{}", dst, lhs, src)]
    Shr {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "shl {},{},{}", dst, lhs, src)]
    Shl {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "ushr {},{},{}", dst, lhs, src)]
    UShr {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "eq {},{},{}", dst, lhs, src)]
    Eq {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "neq {},{},{}", dst, lhs, src)]
    NEq {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "greater {},{},{}", dst, lhs, src)]
    Greater {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "greatereq {},{},{}", dst, lhs, src)]
    GreaterEq {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "less {},{},{}", dst, lhs, src)]
    Less {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "lesseq {},{},{}", dst, lhs, src)]
    LessEq {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "load_global {},{}", dst, name)]
    LoadGlobal {
        dst: VirtualRegister,
        name: VirtualRegister,
    },
    #[display(fmt = "jmp [%{}]", dst)]
    Jump { dst: u32 },
    #[display(fmt = "jmp_cond {}, [%{}],[%{}]", cond, if_true, if_false)]
    JumpConditional {
        cond: VirtualRegister,
        if_true: u32,
        if_false: u32,
    },
    #[display(fmt = "iterator_open {},{}", dst, iterable)]
    // iterator = iteratorFor(iterable)
    IteratorOpen {
        dst: VirtualRegister,
        iterable: VirtualRegister,
    },
    #[display(fmt = "iterator_next {},{},{},{}", next, done, value, iterator)]
    // next = iterator.next();done = next.done;value = next.value;
    IteratorNext {
        next: VirtualRegister,
        done: VirtualRegister,
        value: VirtualRegister,
        iterator: VirtualRegister,
    },
    #[display(fmt = "load_up {},{}", dst, up)]
    LoadUp { dst: VirtualRegister, up: u32 },
    #[display(fmt = "get_by_id {},{},{}", dst, base, id)]
    GetById {
        dst: VirtualRegister,
        base: VirtualRegister,
        id: VirtualRegister,
        fdbk: u32,
    },
    #[display(fmt = "put_by_id {},{},{}", val, base, id)]
    PutById {
        val: VirtualRegister,
        base: VirtualRegister,
        id: VirtualRegister,
    },
    #[display(fmt = "get_by_val {},{},{}", dst, base, val)]
    GetByVal {
        dst: VirtualRegister,
        base: VirtualRegister,
        val: VirtualRegister,
    },
    #[display(fmt = "put_by_val {},{},{}", src, base, val)]
    PutByVal {
        src: VirtualRegister,
        base: VirtualRegister,
        val: VirtualRegister,
    },
    #[display(fmt = "safepoint")]
    Safepoint,
    #[display(fmt = "loop_hint")]
    LoopHint { fdbk: u32 },
    #[display(fmt = "load_this {}", dst)]
    LoadThis { dst: VirtualRegister },
    #[display(fmt = "new_object {}", dst)]
    NewObject { dst: VirtualRegister },
    #[display(fmt = "construct {},{},{}-{}", dst, obj, begin, end)]
    Construct {
        dst: VirtualRegister,
        obj: VirtualRegister,
        begin: VirtualRegister,
        end: VirtualRegister,
    },
    #[display(fmt = "construct {},{}", dst, obj)]
    ConstructNoArgs {
        dst: VirtualRegister,
        obj: VirtualRegister,
    },
}
#[rustfmt::skip]
pub static INS_NAME: [&'static str; 44] = [
    "mov", 
    "load_i32", 
    "new_generator_func", 
    "close_env",
    "close_env_no_args",
    "call",
    "call_no_args",
    "tail_call",
    "yield",
    "return",
    "await",
    "try_catch",
    "throw",
    "add",
    "sub",
    "div",
    "mul",
    "mod",
    "concat",
    "shr",
    "shl",
    "ushr",
    "eq",
    "neq",
    "greater",
    "greatereq",
    "less",
    "lesseq",
    "load_global",
    "jmp",
    "jmp_cond",
    "iterator_open",
    "iterator_next",
    "load_up",
    "get_by_id",
    "put_by_id",
    "get_by_val",
    "put_by_val",
    "safepoint",
    "loophint",
    "load_this",
    "new_object",
    "construct",
    "construct_no_args"
];

#[derive(Hash, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
    Eq,
    NEq,
    Greater,
    GreaterEq,
    Less,
    LessEq,
    UShr,
    Shr,
    Shl,
}

impl Ins {
    pub fn name(self) -> &'static str {
        INS_NAME[unsafe { std::mem::transmute::<_, u64>(std::mem::discriminant(&self)) } as usize]
    }
    pub fn discriminant(self) -> usize {
        unsafe { std::mem::transmute::<_, u64>(std::mem::discriminant(&self)) as usize }
    }
    /// Returns lhs and rhs registers, and hashable `BinaryOp`.
    pub fn to_binary(&self) -> Option<(VirtualRegister, BinaryOp, VirtualRegister)> {
        use Ins::*;
        match *self {
            Add { lhs, src, .. } => Some((lhs, BinaryOp::Add, src)),
            Sub { lhs, src, .. } => Some((lhs, BinaryOp::Sub, src)),
            Div { lhs, src, .. } => Some((lhs, BinaryOp::Div, src)),
            Mul { lhs, src, .. } => Some((lhs, BinaryOp::Mul, src)),
            Mod { lhs, src, .. } => Some((lhs, BinaryOp::Mod, src)),
            Eq { lhs, src, .. } => Some((lhs, BinaryOp::Eq, src)),
            NEq { lhs, src, .. } => Some((lhs, BinaryOp::NEq, src)),
            Greater { lhs, src, .. } => Some((lhs, BinaryOp::Greater, src)),
            GreaterEq { lhs, src, .. } => Some((lhs, BinaryOp::GreaterEq, src)),
            Less { lhs, src, .. } => Some((lhs, BinaryOp::Less, src)),
            LessEq { lhs, src, .. } => Some((lhs, BinaryOp::LessEq, src)),
            UShr { lhs, src, .. } => Some((lhs, BinaryOp::UShr, src)),
            Shr { lhs, src, .. } => Some((lhs, BinaryOp::Shr, src)),
            Shl { lhs, src, .. } => Some((lhs, BinaryOp::Shl, src)),
            _ => None,
        }
    }
    pub fn is_final(&self) -> bool {
        use Ins::*;
        match self {
            Return { .. } => true,
            Jump { .. } => true,
            JumpConditional { .. } => true,
            _ => false,
        }
    }
    pub fn get_defs(&self) -> Vec<VirtualRegister> {
        let mut set = Vec::new();
        macro_rules! r {
            ($x: expr) => {{
                if $x.is_local() {
                    set.push($x);
                }
            }};
        }
        use Ins::*;
        match *self {
            Mov { dst, .. } => r!(dst),
            Add { dst, .. }
            | Sub { dst, .. }
            | Div { dst, .. }
            | Mul { dst, .. }
            | Greater { dst, .. }
            | GreaterEq { dst, .. }
            | Less { dst, .. }
            | LessEq { dst, .. }
            | Eq { dst, .. }
            | NEq { dst, .. }
            | LoadI32 { dst, .. }
            | NewGeneratorFunction { dst, .. }
            | Call { dst, .. }
            | CallNoArgs { dst, .. }
            | Yield { dst, .. }
            | Await { dst, .. }
            | TryCatch { reg: dst, .. }
            | Concat { dst, .. }
            | Shr { dst, .. }
            | Shl { dst, .. }
            | UShr { dst, .. }
            | LoadUp { dst, .. }
            | LoadGlobal { dst, .. }
            | IteratorOpen { dst, .. }
            | GetById { dst, .. }
            | GetByVal { dst, .. }
            | LoadThis { dst, .. }
            | Mod { dst, .. } => r!(dst),

            IteratorNext {
                next, done, value, ..
            } => {
                r!(next);
                r!(done);
                r!(value);
            }
            ConstructNoArgs { dst, .. } | Construct { dst, .. } => r!(dst),
            CloseEnv { dst, .. } | CloseEnvNoArgs { dst, .. } => r!(dst),
            _ => (),
        }
        set
    }

    pub fn get_uses(&self) -> Vec<VirtualRegister> {
        let mut set = Vec::new();
        macro_rules! r {
            ($x: expr) => {{
                if $x.is_local() {
                    set.push($x);
                }
            }};
            ($($x: expr),*) => {
                {$(r!($x);)*}
            }
        }
        use Ins::*;
        match *self {
            Construct { obj, .. } | Ins::ConstructNoArgs { obj, .. } => r!(obj),
            Mov { src, .. } => r!(src),
            NewGeneratorFunction { src, .. } => r!(src),
            CloseEnvNoArgs { function, .. } | CloseEnv { function, .. } => r!(function),
            Call { function, this, .. } | CallNoArgs { function, this, .. } => r!(function, this),
            Ins::Yield { res, .. } => r!(res),
            Throw { src } => r!(src),
            Add { lhs, src, .. }
            | Sub { lhs, src, .. }
            | Div { lhs, src, .. }
            | Mul { lhs, src, .. }
            | Mod { lhs, src, .. }
            | Shr { lhs, src, .. }
            | Shl { lhs, src, .. }
            | UShr { lhs, src, .. }
            | Eq { lhs, src, .. }
            | NEq { lhs, src, .. }
            | Greater { lhs, src, .. }
            | GreaterEq { lhs, src, .. }
            | Less { lhs, src, .. }
            | LessEq { lhs, src, .. }
            | Concat { lhs, src, .. } => r!(lhs, src),
            Ins::IteratorNext { iterator, .. } => r!(iterator),
            IteratorOpen { iterable, .. } => r!(iterable),
            GetById { base, id, .. } => r!(base, id),
            GetByVal { base, val, .. } => r!(base, val),
            PutById { base, id, .. } => r!(base, id),
            PutByVal { base, val, .. } => r!(base, val),
            Ins::JumpConditional { cond, .. } => r!(cond),
            Return { val } => r!(val),
            Await { on, .. } => r!(on),
            _ => (),
        }
        set
    }

    pub fn replace_reg(&mut self, from: VirtualRegister, to: VirtualRegister) {
        macro_rules! r {
            ($x: expr) => {{
                if *$x == from {
                    *$x = to;
                }
            }};
            ($($x: expr),*) => {
                {$(r!($x);)*}
            }
        }
        use Ins::*;
        match self {
            NewObject { dst } => r!(dst),
            ConstructNoArgs { dst, obj, .. } | Construct { dst, obj, .. } => r!(dst, obj),
            Mov { dst, src } => r!(dst, src),
            Add { dst, lhs, src, .. }
            | Sub { dst, lhs, src, .. }
            | Div { dst, lhs, src, .. }
            | Mul { dst, lhs, src, .. }
            | Mod { dst, lhs, src, .. }
            | Shr { dst, lhs, src, .. }
            | Shl { dst, lhs, src, .. }
            | UShr { dst, lhs, src, .. }
            | Greater { dst, lhs, src, .. }
            | GreaterEq { dst, lhs, src, .. }
            | Less { dst, lhs, src, .. }
            | LessEq { dst, lhs, src, .. }
            | Eq { dst, lhs, src, .. }
            | NEq { dst, lhs, src, .. } => r!(dst, lhs, src),
            LoadGlobal { dst, .. } => r!(dst),
            Yield { dst, res } => r!(dst, res),
            Return { val } => r!(val),
            Await { dst, on } => r!(dst, on),
            JumpConditional { cond, .. } => r!(cond),
            IteratorOpen { dst, iterable } => r!(dst, iterable),
            IteratorNext {
                done,
                next,
                value,
                iterator,
            } => r!(next, done, value, iterator),
            LoadUp { dst, .. } => r!(dst),
            GetById { dst, base, id, .. } => r!(dst, base, id),
            PutById { val, base, id, .. } => r!(val, base, id),
            GetByVal { dst, base, val, .. } => r!(dst, base, val),
            PutByVal { src, base, val } => r!(src, base, val),
            Call {
                function,
                dst,
                this,
                ..
            }
            | Ins::CallNoArgs {
                function,
                dst,
                this,
                ..
            } => r!(function, dst, this),
            NewGeneratorFunction { dst, src } => r!(dst, src),
            CloseEnvNoArgs { dst, function, .. } | CloseEnv { dst, function, .. } => {
                r!(dst, function)
            }
            Throw { src } => r!(src),
            TryCatch { reg, .. } => r!(reg),
            LoadThis { dst } => r!(dst),
            _ => (),
        }
    }

    pub fn branch_targets(&self) -> Vec<u32> {
        match &self {
            Ins::Jump { dst } => vec![*dst],
            Ins::JumpConditional {
                cond: _,
                if_true,
                if_false,
            } => vec![*if_true, *if_false],
            Ins::TryCatch { try_, catch, .. } => vec![*try_, *catch],
            _ => vec![],
        }
    }

    pub fn is_jump(self) -> bool {
        match self {
            Ins::Jump { .. } => true,
            _ => false,
        }
    }
}
