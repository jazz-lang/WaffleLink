use bytecode::*;
use heap::*;
use interpreter::callframe::*;
use jit::*;
use object::*;
use value::*;
use virtual_register::*;
use wafflelink::*;
pub extern "C" fn foo(cf: &mut CallFrame) -> WaffleResult {
    assert!(cf.this.is_int32());
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
        Ins::Add(
            virtual_register_for_local(0),
            VirtualRegister::new_constant_index(0),
            VirtualRegister::new_argument(0),
        ),
        Ins::Return(virtual_register_for_local(0)),
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
}
