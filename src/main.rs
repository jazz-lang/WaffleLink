extern crate wafflelink;
use opcode::Opcode::*;
use value::*;
use wafflelink::module::*;
use wafflelink::object::*;
use wafflelink::*;
fn foo() {
    for _ in 0..3200 {
        let x = HMap::new_empty(2).to_heap();
        x.value_mut().set(Value::new_int(1), Value::new_int(2));
    }
}
fn main() {
    let x = false;
    VM.register_thread(&x);
    let t = std::time::Instant::now();

    simple_logger::init().unwrap();
    let map = HMap::new_empty(4).to_heap();
    let module = Module::new_empty(1, &[Value::new_int(4), Value::new_int(3)]).to_heap();
    let code = vec![Constant(0, 0), Constant(1, 1), Add(0, 1, 0), Ret(0)];
    println!("{:p} {:p}", &module, module.raw());
    foo();
    map.value_mut().set(Value::new_int(3), Value::undefined());
    println!("done in {}ms", t.elapsed().as_millis());
    VM.collect();
    map.value_mut().set(
        Value::new_int(4),
        Value::from(WaffleArray::new(Value::new_int(42), 10).to_heap()),
    );
    map.value_mut().set(Value::new_int(64), Value::from(module));
    map.value_mut().set(Value::new_int(5), Value::undefined());
    VM.collect();
    println!("{:p}", &module);
}
