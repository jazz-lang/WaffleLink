extern crate wafflelink;
use value::*;
use wafflelink::object::*;
use wafflelink::*;
fn main() {
    let x = false;
    VM.register_thread(&x);
    simple_logger::init().unwrap();

    let map = HMap::new_empty(0);
    map.value_mut().set(Value::new_int(0), Value::new_int(1));
    map.value_mut().set(Value::new_int(1), Value::new_int(2));
    map.value_mut().set(Value::new_int(2), Value::new_int(3));
    VM.collect();
    println!(
        "{}",
        map.value().getp(Value::new_int(1)).unwrap().to_int32()
    );
}
