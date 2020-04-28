use jlight::bytecode::BytecodeAssembler;
use jlight::common::ptr::Ptr;

use jlight::runtime;
use runtime::cell::*;
use runtime::frame::Frame;
use runtime::process::*;
use runtime::symbol::*;
use runtime::value::*;
fn main() {
    let mut f = Frame::new(Value::empty(), Value::empty());
    let mut v = local_data().allocate_string("Hello,World!", &mut f);
    let x = local_data().allocate_string("x", &mut f);
    let z = local_data().allocate_string("y", &mut f);
    let y = local_data().allocate_string("length", &mut f);
    let mut slot = Slot::new();
    v.insert(Symbol::new_value(x), &mut slot);
    assert!(slot.store(Value::new_int(42)));
    let mut slot = Slot::new();
    //v.insert(Symbol::new_value(y), &mut slot);
    //assert!(slot.store(Value::new_int(3)));
    let mut slot = Slot::new();

    v.lookup(Symbol::new_value(y), &mut slot);
    println!(" {}", slot.value().to_int32());
}
