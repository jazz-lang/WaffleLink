use bytecompiler::*;
use value::*;
use wafflelink::*;

use frontend::parser::*;
use frontend::reader::*;

fn main() {
    let x = false;
    let mut vm = VM::new(&x);
    vm.template_jit = true;
    vm.disasm = true;
    wafflelink::LOG.store(!true, std::sync::atomic::Ordering::Relaxed);
    set_vm(&vm);
    runtime::initialize();
    let reader = Reader::from_string(
        "
        
function fac(x) {

    if x < 2 {
        return 1
    }
    return fac(x - 1) * x
}
print(fac(5))
        ",
    );
    let mut ast = vec![];
    let mut p = Parser::new(reader, &mut ast);
    if let Err(e) = p.parse() {
        eprintln!("{}", e);
        return;
    }
    let code = match compile(&ast) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let mut b = String::new();
    code.dump(&mut b).unwrap();
    println!("{}", b);
    let fun = function::Function::new(&mut get_vm().heap, code, "<main>");
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
