use bytecode::*;
use jit::*;
use value::*;
use virtual_register::*;
use wafflelink::*;
fn main() {
    let mut cb = CodeBlock::new();
    cb.constants.push(Value::new_int(2));
    cb.constants.push(Value::new_int(3));
    cb.num_vars = 1;
    cb.callee_locals = 1;
    cb.instructions = vec![
        Ins::Enter,
        Ins::Add(
            VirtualRegister::new_constant_index(0),
            VirtualRegister::new_constant_index(1),
            virtual_register_for_local(0),
        ),
        Ins::Return(virtual_register_for_local(0)),
    ];
    let mut jit = JIT::new(&cb);
    jit.compile_without_linking();
    jit.link();
    jit.disasm();
}
