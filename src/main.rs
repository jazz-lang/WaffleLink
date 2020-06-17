extern crate wafflelink;
use wafflelink::object::*;
use wafflelink::tagged::*;
use wafflelink::*;
fn foo() {
    let _obj = VM
        .state
        .heap
        .allocate(WaffleType::Object, std::mem::size_of::<WaffleObject>())
        .unwrap();
    VM.collect();
    println!("{:?} {:p}", _obj.type_of(), &_obj);
}
fn main() {
    simple_logger::init().unwrap();
    let x = false;
    VM.register_thread(&x);
    foo();
    VM.collect();
}
