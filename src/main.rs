use bytecompiler::*;
use frontend::parser::*;
use frontend::reader::*;
use std::path::PathBuf;
use structopt::StructOpt;
use value::*;
use wafflelink::*;
#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(long = "dumpBytecode", help = "Dump bytecode")]
    dump_bc: bool,
    #[structopt(
        long = "disassemble",
        help = "Dump machine disassembly if JIT is enabled"
    )]
    disasm: bool,
    #[structopt(
        long = "useJIT",
        help = "Enable baseline JIT compiler",
        default_value = "1"
    )]
    template_jit: u8,
    #[structopt(long = "useOPTJIT", help = "Enable optimizing JIT compiler")]
    opt_jit: bool,
    #[structopt(
        long = "useTracingJIT",
        help = "Enable tracing JIT compiler, works only when useJIT=false and useOPTJIT=false"
    )]
    tracing_jit: bool,
    #[structopt(
        long = "jitThreshold",
        help = "Set threshold before OSR to JIT",
        default_value = "100"
    )]
    jit_threshold: usize,
    #[structopt(short, long = "verbose")]
    verbose: bool,
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    #[structopt(long = "verboseAlloc", help = "Verbose log when allocating")]
    verbose_alloc: bool,
}

fn main() {
    let opt: Opts = Opts::from_args();
    let x = false;
    let mut vm = VM::new(&x);
    vm.template_jit = opt.template_jit == 1;
    vm.disasm = opt.disasm;
    vm.dump_bc = opt.dump_bc;
    vm.verbose_alloc = opt.verbose_alloc;
    vm.jit_threshold = opt.jit_threshold as _;
    wafflelink::LOG.store(opt.verbose, std::sync::atomic::Ordering::Relaxed);
    set_vm(&*vm);
    runtime::initialize();
    let reader = Reader::from_file(opt.input.as_os_str().to_str().unwrap()).unwrap();
    let mut ast = vec![];
    let mut p = Parser::new(reader, &mut ast);
    if let Err(e) = p.parse() {
        eprintln!("{}", e);
        return;
    }
    let (m, code) = match compile(&ast) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    if vm.dump_bc {
        println!("CodeBlock for global script:");
        let mut b = String::new();
        code.dump(&mut b).unwrap();
        println!("{}", b);
    }
    let mut fun = function::Function::new(&mut get_vm().heap, code, "<main>");
    fun.module = Some(m);
    let start = std::time::Instant::now();
    let res = fun.execute(Value::undefined(), &[]);
    let e = start.elapsed();
    println!("executed code in {}ms or {}ns", e.as_millis(), e.as_nanos());
    if res.is_error() {
        println!("Error");
        runtime::print_val(res.value());
    } else {
        runtime::print_val(res.value());
    }
}

/*
pub extern "C" fn foo(cf: &mut CallFrame) -> WaffleResult {
    assert!(cf.this.is_int32());
    let arg = cf.get_register(VirtualRegister::new_argument(0));
    if cf.this != arg {
        panic!();
    }
    WaffleResult::okay(Value::new_int(322))
}
fn main() {
    simple_logger::init().unwrap();
    let x = false;
    let vm = VM::new(&x);
    //vm.log = true;
    set_vm(&vm);
    let mut heap = Heap::new(&x);
    let func = function::Function::new_native(&mut heap, foo);
    let mut cb = Box::new(CodeBlock::new());
    cb.constants.push(Value::new_int(2));
    cb.constants.push(Value::from(func.cast()));
    cb.num_vars = 1;
    cb.callee_locals = 7;
    cb.metadata = vec![{
        let mut meta = OpcodeMetadata::new();
        meta.arith_profile.lhs_saw_int32();
        meta.arith_profile.rhs_saw_int32();
        meta
    }];
    cb.instructions = vec![
        Ins::Move(
            virtual_register_for_local(0),
            VirtualRegister::new_constant_index(0), // int32 2 as 'this' value
        ),
        Ins::Move(
            virtual_register_for_local(1),
            VirtualRegister::new_constant_index(1), // function
        ),
        Ins::Move(
            virtual_register_for_local(2),
            VirtualRegister::new_constant_index(0), // int32 2 as argument
        ),
        Ins::Call(
            virtual_register_for_local(0), //dest
            virtual_register_for_local(0), // this
            virtual_register_for_local(1), // callee
            1,                             // argc
        ),
        Ins::Return(virtual_register_for_local(0)),
    ];
    let mut jit = JIT::new(&cb);
    println!("{:p}", &cb);
    jit.compile_without_linking();
    jit.link();
    jit.disasm();
    let mut cf = Box::new(CallFrame::new(&[Value::new_int(4)], 4));

    cf.code_block = Some(Ref {
        ptr: std::ptr::NonNull::new((&*cb) as *const CodeBlock as *mut CodeBlock).unwrap(),
    });
    let f: extern "C" fn(&mut CallFrame) -> WaffleResult =
        unsafe { std::mem::transmute(jit.link_buffer.code) };
    let res = f(&mut cf);
    println!("{}", res.value().to_int32());
}
*/
