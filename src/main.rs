extern crate cgc_single_threaded as cgc;
use cgc::api::*;
use cgc::heap::*;
use waffle2::bytecode::virtual_reg::*;
macro_rules! foo {
    ($x: expr) => {
        1 + $x
    };
}

fn main() {
    let x = foo!(4);
}
