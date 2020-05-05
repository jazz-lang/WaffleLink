extern crate cgc_single_threaded as cgc;
use cgc::api::*;
use cgc::heap::*;
use waffle2::bytecode::virtual_reg::*;
fn main() {
    let r = VirtualRegister::constant(255);
    println!("{:?}", r);
    println!("{}", r.to_constant());
}
