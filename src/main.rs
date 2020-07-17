use bigint::*;
use bytecode::*;
use gc::*;
use heap::*;
use interpreter::callframe::*;
use jit::*;
use num_bigint::*;
use object::*;
use value::*;
use virtual_register::*;
use wafflelink::*;

fn main() {
    let vm = VM::new();
    set_vm(&vm);
    simple_logger::init().unwrap();
    let mut cb = CodeBlock::new();
    cb.constants.push(Value::new_int(2));

    cb.num_vars = 1;
    cb.callee_locals = 7;
    cb.metadata = vec![{ OpcodeMetadata::new() }, OpcodeMetadata::new()];
    cb.instructions = vec![
        Ins::Try(2),
        Ins::JLess(
            virtual_register_for_local(0),
            virtual_register_for_local(1),
            -1,
        ),
        Ins::Return(virtual_register_for_local(2)),
        Ins::Catch(virtual_register_for_local(0)),
        //Ins::Jump(-1),
    ];
    let mut jit = JIT::new(&cb);
    println!("{:p}", &cb);
    jit.compile_without_linking();
    jit.link();
    jit.disasm();
    /*let mut cf = CallFrame::new(&[Value::new_int(4)], 3);
    cf.code_block = Some(Ref { ptr: &cb });
    let f: extern "C" fn(&mut CallFrame) -> WaffleResult =
        unsafe { std::mem::transmute(jit.link_buffer.code) };
    println!("Invoking for first time, should generate IC snippet");
    let res = f(&mut cf).value().to_int32();
    println!("Result: {}", res);
    println!(
        "Second time invoke result: {}",
        f(&mut cf).value().to_int32()
    );
    //jit.disasm();*/
}
