use jlight::bytecode::BytecodeAssembler;
use jlight::common::ptr::Ptr;
use jlight::interpreter::dispatch;
use jlight::runtime;
use runtime::frame::Frame;
use runtime::value::*;
fn main() {
    let mut bc = BytecodeAssembler::new();
    bc.lda_int(42);
    bc.ret();
    let f = Ptr::null();

    let frame = Frame::new(Value::empty(), Value::empty());

    match dispatch(
        frame,
        f,
        Ptr {
            raw: bc.code.as_ptr() as *mut u8,
        },
    ) {
        Ok(val) => println!("{}", val.to_number()),
        _ => panic!(),
    }
}
