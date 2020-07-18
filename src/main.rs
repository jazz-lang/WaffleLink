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
    let x = false;
    let mut vm = VM::new(&x);
    vm.log = true;
    set_vm(&vm);
    simple_logger::init().unwrap();
    let mut cb = Box::new(CodeBlock::new());
    cb.constants.push(Value::new_int(2));

    cb.num_vars = 1;
    cb.callee_locals = 7;
    cb.metadata = vec![
        {
            let mut meta = OpcodeMetadata::new();
            // meta.arith_profile.observe_lhs(Value::new_int(0));
            // meta.arith_profile.observe_rhs(Value::new_int(0));
            meta
        },
        OpcodeMetadata::new(),
    ];
    cb.instructions = vec![
        Ins::Add(
            virtual_register_for_local(0),
            VirtualRegister::new_argument(0),
            virtual_register_for_local(1),
        ),
        Ins::Return(virtual_register_for_local(1)),
    ];
    let mut jit = JIT::new(&cb);
    println!("{:p}", &cb);
    jit.compile_without_linking();
    jit.link();
    jit.disasm();
    let mut cf = Box::new(CallFrame::new(&[Value::new_int(4)], 3));
    cf.regs = vec![Value::new_int(42)].into_boxed_slice();
    cf.code_block = Some(Ref { ptr: &*cb });
    let f: extern "C" fn(&mut CallFrame) -> WaffleResult =
        unsafe { std::mem::transmute(jit.link_buffer.code) };
    let res = f(&mut cf);
    println!("{}", res.value().to_int32());
    println!("{}", cb.num_args);
    //jit.disasm();*/
}
