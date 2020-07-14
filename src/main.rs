use bigint::*;
use bytecode::*;
use gc::*;
use heap::*;
use jit::*;
use num_bigint::*;
use object::*;
use value::*;
use virtual_register::*;
use wafflelink::*;

fn main() {
    simple_logger::init().unwrap();
    let start = false;
    let mut heap = Heap::new(&start);
    let array = Array::new(&mut heap, 10, Value::new_int(42));
    assert!(array.get_at(3).is_int32());
    let array = Array::new(&mut heap, 10, Value::new_int(4));
    assert!(array.get_at(3).is_int32());
    let top = false;
    heap.collect(Address::from_ptr(&top));
    /*let vm = VM::new();
    set_vm(&vm);
    simple_logger::init().unwrap();
    let mut cb = CodeBlock::new();
    cb.constants.push(Value::new_int(2));
    cb.constants.push(Value::new_int(5));
    cb.num_vars = 1;
    cb.callee_locals = 7;
    cb.metadata = vec![
        OpcodeMetadata::new(),
        {
            let mut meta = OpcodeMetadata::new();
            meta.arith_profile.lhs_saw_number();
            meta.arith_profile.rhs_saw_number();
            meta
        },
        OpcodeMetadata::new(),
    ];
    cb.instructions = vec![
        Ins::Enter,
        Ins::Safepoint,
        Ins::Div(
            VirtualRegister::new_constant_index(1),
            VirtualRegister::new_constant_index(0),
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
    jit.disasm();*/
}
