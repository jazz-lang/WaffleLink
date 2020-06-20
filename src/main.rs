extern crate wafflelink;
use value::*;
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
    let x = false;
    //VM.register_thread(&x);
    //simple_logger::init().unwrap();

    {
        let map = HMap::new_empty(0);
        map.value_mut().set(Value::new_int(0), Value::new_int(1));
        map.value_mut().set(Value::new_int(1), Value::new_int(2));
        map.value_mut().set(Value::new_int(2), Value::new_int(3));
        map.value_mut().set(
            Value::new_int(3),
            Value::from(WaffleObject::new_empty(4).to_heap()),
        );
        foo();
    }
    println!("done in {}ms", t.elapsed().as_millis());
}
