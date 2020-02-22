use cell::*;
use instruction::*;
use module::*;
use process::*;
use value::*;
use waffle::bytecode::*;
use waffle::heap::cms::atomic_list::AtomicList;
use waffle::runtime::*;
use waffle::util::arc::Arc;
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
