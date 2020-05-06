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
    #[display(fmt = "close_env {}, {}-{}", function, begin, end)]
    // function.env = registers[begin..end]
    CloseEnv {
        function: VirtualRegister,
        begin: VirtualRegister,
        end: VirtualRegister,
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
}
