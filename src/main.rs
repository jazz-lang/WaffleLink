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
    let mut m = Arc::new(Module::new("Main"));
    let code = basicblock::BasicBlock::new(vec![Instruction::Gc, Instruction::Return(None)], 0);
    let func = Function {
        upvalues: vec![],
        name: Arc::new("main".to_owned()),
        module: m.clone(),
        code: Arc::new(vec![code]),
        native: None,
        argc: 0,
    };
    let value = RUNTIME.state.allocate_fn(func);
    let proc = Process::from_function(value, &config::Config::default()).unwrap();
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!1");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!2");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!3");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!4");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!5");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!6");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!7");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!9");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!11");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!22");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!33");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!44");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!55");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!66");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!77");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!99");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!111");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!222");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!333");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!444");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!555");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!666");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!777");
    m.globals.push(s);
    let s = Process::allocate_string(&proc, &RUNTIME.state, "Wooooow!9999");
    m.globals.push(s);
    RUNTIME.schedule_main_process(proc.clone());
    RUNTIME.start_pools();
    //println!("{}", proc.is_terminated());
    m.globals.pop();
    m.globals.remove(3);
    m.globals.remove(17);
    m.globals.remove(7);
    let x = std::time::Instant::now();
    Process::do_gc(&proc);
    m.globals.clear();
    Process::do_gc(&proc);
    let e = x.elapsed();
    println!(
        "{}ns {}micros {}ms",
        e.as_nanos(),
        e.as_micros(),
        e.as_millis()
    )
}
