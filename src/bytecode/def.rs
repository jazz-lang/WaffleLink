use super::virtual_reg::*;

#[derive(Copy, Clone)]
pub enum Ins {
    // dst = src
    Mov {
        dst: VirtualRegister,
        src: VirtualRegister,
    },
    // dst = imm
    LoadI32 {
        dst: VirtualRegister,
        imm: i32,
    },
    // dst = new Generator(src)
    NewGeneratorFunction {
        dst: VirtualRegister,
        src: VirtualRegister,
    },
    // function.env = registers[begin..end]
    CloseEnv {
        function: VirtualRegister,
        begin: VirtualRegister,
        end: VirtualRegister,
    },
    // dst = function.apply(this,registers[begin..end])
    Call {
        dst: VirtualRegister,
        function: VirtualRegister,
        this: VirtualRegister,
        begin: VirtualRegister,
        end: VirtualRegister,
    },
    // dst = yield res
    Yield {
        dst: VirtualRegister,
        res: VirtualRegister,
    },
    // return val
    Return {
        val: VirtualRegister,
    },
    // dst = await on
    Await {
        dst: VirtualRegister,
        on: VirtualRegister,
    },
    TryCatch {
        // try block, we jump to it immediatly
        try_: u32,
        // catch block
        catch: u32,
        // register where to store exception if thrown.
        reg: VirtualRegister,
    },
    Throw {
        src: VirtualRegister,
    },
    Add {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    Sub {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    Div {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    Mul {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    Mod {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    Shr {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    Shl {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    UShr {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    Eq {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    NEq {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    Greater {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    GreaterEq {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    Less {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    LessEq {
        dst: VirtualRegister,
        lhs: VirtualRegister,
        src: VirtualRegister,
        fdbk: u32,
    },
    LoadGlobal {
        dst: VirtualRegister,
        name: VirtualRegister,
    },
    Jump {
        dst: u32,
    },
    JumpConditional {
        cond: VirtualRegister,
        if_true: u32,
        if_false: u32,
    },
    // iterator = iteratorFor(iterable)
    IteratorOpen {
        dst: VirtualRegister,
        iterable: VirtualRegister,
    },
    // next = iterator.next();done = next.done;value = next.value;
    IteratorNext {
        next: VirtualRegister,
        done: VirtualRegister,
        value: VirtualRegister,
        iterator: VirtualRegister,
    },
    LoadUp {
        dst: VirtualRegister,
        up: u32,
    },
    GetById {
        dst: VirtualRegister,
        base: VirtualRegister,
        id: VirtualRegister,
        fdbk: u32,
    },
    PutById {
        val: VirtualRegister,
        base: VirtualRegister,
        id: VirtualRegister,
    },
    GetByVal {
        dst: VirtualRegister,
        base: VirtualRegister,
        val: VirtualRegister,
    },
    PutByVal {
        src: VirtualRegister,
        base: VirtualRegister,
        val: VirtualRegister,
    },
    Safepoint,
    LoopHint {
        fdbk: u32,
    },
}
