extern crate wafflelink;
use wafflelink::object::WaffleTypeHeader;

fn main() {
    println!("{}", std::mem::size_of::<WaffleTypeHeader>());
}
