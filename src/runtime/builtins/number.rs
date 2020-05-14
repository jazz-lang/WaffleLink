use crate::interpreter::Return;
use crate::runtime::*;
use cell::*;
use std::fmt::*;
use value::*;

pub fn times(rt: &mut Runtime, this: Value, args: &[Value]) -> Return {
    if args.is_empty() {
        return Return::Error(Value::from(
            rt.allocate_string("times: expected at least one argument."),
        ));
    }

    let x = this.to_number().floor() as i64;
    for i in 0..x {
        match rt.call(args[0], Value::number(i as f64), &[Value::number(i as _)]) {
            Err(error) => return Return::Error(error),
            _ => {}
        }
    }
    Return::Return(Value::undefined())
}

pub fn initialize(rt: &mut Runtime) {
    let f = native_fn!(rt, "times", times);
    rt.number_prototype.put_named(f, "times");
    rt.globals.insert(
        "Number".to_owned(),
        Value::from(rt.number_prototype.to_heap()),
    );
}
