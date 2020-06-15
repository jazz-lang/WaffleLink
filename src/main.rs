extern crate wafflelink;
use wafflelink::gc::*;
use wafflelink::object::WaffleTypeHeader;
use wafflelink::object::*;
use wafflelink::value::*;

fn main() {
    simple_logger::init().unwrap();
    let top = false;
    let mut local = LocalAllocator::new(&top);

    let arr = local.allocate_array(3);
    *arr.value_mut().at_ref_mut(1) = Value::new_int(42);
    println!("{}", arr.value().len());
    println!("{}", arr.value().at(1).as_int32());

    unsafe {
        local.gc_collect();
        local.gc_collect();
    }
}
