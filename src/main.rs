extern crate cgc_single_threaded as cgc;
use cgc::api::*;
use cgc::heap::*;
use waffle2::bytecode::virtual_reg::*;

fn main() {
    println!("{}", 0x7FFFFFFF - 0x40000000);
}
