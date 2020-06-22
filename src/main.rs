extern crate wafflelink;
use opcode::Opcode::*;
use value::*;
use wafflelink::module::*;
use wafflelink::object::*;
use wafflelink::*;
fn foo() {
    for _ in 0..3200 {
        let x = HMap::new_empty(2);
        x.value_mut().set(Value::new_int(1), Value::new_int(2));
    }
}
fn main() {
    let t = std::time::Instant::now();
    //VM.register_thread(&x);
    //simple_logger::init().unwrap();

    let module = Module::new_empty(1, &[Value::new_int(4), Value::new_int(3)]);
    let code = vec![Constant(0, 0), Constant(1, 1), Add(0, 1, 0), Ret(0)];
    println!("done in {}ms", t.elapsed().as_millis());
}
