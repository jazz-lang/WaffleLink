use bytecode::*;
use jit::*;
use value::*;
use virtual_register::*;
use wafflelink::*;
fn main() {
    let vm = VM::new();
    set_vm(&vm);
    simple_logger::init().unwrap();
    let mut cb = CodeBlock::new();
    cb.constants.push(Value::new_int(2));
    cb.constants.push(Value::new_int(4));
    cb.num_vars = 1;
    cb.callee_locals = 7;
    cb.metadata = vec![
        OpcodeMetadata::new(),
        {
            let mut meta = OpcodeMetadata::new();
            //meta.arith_profile.lhs_saw_int32();
            //meta.arith_profile.rhs_saw_int32();
            meta
        },
        OpcodeMetadata::new(),
    ];
    cb.instructions = vec![
        Ins::Enter,
        Ins::Add(
            VirtualRegister::new_constant_index(0),
            VirtualRegister::new_constant_index(1),
            virtual_register_for_local(7),
        ),
        Ins::Return(virtual_register_for_local(7)),
    ];
    let mut jit = JIT::new(&cb);
    println!("{:p}", &cb);
    jit.compile_without_linking();
    jit.link();
    jit.disasm();
    let f: extern "C" fn() -> Value = unsafe { std::mem::transmute(jit.link_buffer.code) };

    println!("{}", f().to_int32());
    //jit.disasm();
}
