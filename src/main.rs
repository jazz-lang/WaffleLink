use cell::*;
use instruction::*;
use module::*;
use process::*;
use value::*;
use waffle::bytecode::*;
use waffle::heap::cms::atomic_list::AtomicList;
use waffle::runtime::*;
use waffle::util::arc::Arc;

#[allow(unused_macros)]
macro_rules! waffle_asm {
    (
        $(
            const $value: expr;
        )*
        code_start:
        $(
            func $func_name: ident : $argc: expr => {
                $(
                    $block_index: expr => {
                        $($rest: tt)*
                    }
                )*
            }
        )*
    ) => {
    };

    (@ins $bcode: expr,load_int $r0: expr, $i: expr; $($rest: tt)*) => {
        $bcode.push(Instruction::LoadInt($r0 as u16,$i as i32));
        waffle_asm!(@ins $bcode,$($rest: tt)*);
    };
    (@ins $bcode: expr,add $r0: expr,$r1: expr,$r2: expr) => {
        $bcode.push(Instruction::Binary(BinOp::Add,$r0,$r1,$r2));
        waffle_asm!(@ins $bcode,$($rest: tt)*);
    };
    (@ins $bcode: expr,sub $r0: expr,$r1: expr,$r2: expr) => {
        $bcode.push(Instruction::Binary(BinOp::Sub,$r0,$r1,$r2));
        waffle_asm!(@ins $bcode, $($rest: tt)*);
    };
    (@ins $bcode: expr,mul $r0: expr,$r1: expr,$r2: expr) => {
        $bcode.push(Instruction::Binary(BinOp::Mul,$r0,$r1,$r2));
        waffle_asm!(@ins $bcode, $($rest: tt)*);
    };
    (@ins $bcode: expr,div $r0: expr,$r1: expr,$r2: expr) => {
        $bcode.push(Instruction::Binary(BinOp::Div,$r0,$r1,$r2));
        waffle_asm!(@ins $bcode, $($rest: tt)*);
    };
    (@ins $bcode: expr,cmp $cmp_op: ident $r0: expr,$r1: expr,$r2: expr) => {
        $bcode.push(Instruction::Binary(BinOp::$cmp_op,$r0,$r1,$r2));
        waffle_asm!(@ins $bcode, $($rest: tt)*);
    };
    (@ins $bcode: expr,call $r0: expr,$r1: expr,$r2: expr) => {
        $bcode.push(Instruction::Call($r0,$r1,$r2));
        waffle_asm!(@ins $bcode,$($rest: tt)*);
    };
    (@ins $bcode: expr,tail_call $r0: expr,$r1: expr,$r2: expr) => {
        $bcode.push(Instruction::TailCall($r0,$r1,$r2));
        waffle_asm!(@ins $bcode,$($rest: tt)*);
    };
    (@ins $bcode: expr,virtcall $r0: expr,$r1: expr,$r2: expr,$r3: expr) => {
        $bcode.push(Instruction::VirtCall($r0,$r1,$r2,$r3));
        waffle_asm!(@ins $bcode,$($rest: tt)*);
    };
    (@ins $bcode: expr,new $r0: expr,$r1: expr,$r2: expr) => {
        $bcode.push(Instruction::New($r0,$r1,$r2));
        waffle_asm!(@ins $bcode,$($rest:tt)*);
    };
    (@ins $bcode: expr,) => {

    }
}

fn main() {
    //simple_logger::init().unwrap();
    let x = std::time::Instant::now();
    let mut m = Arc::new(Module::new("Main"));
    let code = vec![
        basicblock::BasicBlock::new(vec![Instruction::LoadInt(0, 0), Instruction::Branch(1)], 0),
        basicblock::BasicBlock::new(
            vec![
                Instruction::LoadInt(1, 10000000),
                Instruction::Binary(BinOp::Greater, 2, 1, 0),
                Instruction::ConditionalBranch(2, 2, 3),
            ],
            0,
        ),
        basicblock::BasicBlock::new(
            vec![
                Instruction::LoadInt(1, 1),
                Instruction::Binary(BinOp::Add, 0, 0, 1),
                Instruction::Branch(1),
            ],
            0,
        ),
        basicblock::BasicBlock::new(vec![Instruction::Return(None)], 0),
    ];
    let func = Function {
        upvalues: vec![],
        name: Value::from(RUNTIME.state.intern_string("main".to_owned())),
        module: m.clone(),
        code: Arc::new(code),
        native: None,
        argc: 0,
        md: Default::default(),
    };
    let value = RUNTIME.state.allocate_fn(func);
    let proc = Process::from_function(value, &config::Config::default()).unwrap();
    RUNTIME.schedule_main_process(proc.clone());
    RUNTIME.start_pools();

    m.globals.clear();
    let e = x.elapsed();
    println!(
        "{}ns {}micros {}ms",
        e.as_nanos(),
        e.as_micros(),
        e.as_millis()
    )
}
