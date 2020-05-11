use crate::interpreter::Return;
use crate::runtime::*;
use cell::*;
use std::fmt::*;
use value::*;
pub fn to_string(rt: &mut Runtime, this: Value, _: &[Value]) -> Return {
    let mut res = Value::undefined();

    if this.is_cell() {
        let mut buf = String::new();
        write!(buf, "{{ ").unwrap();
        for (i, (key, prop)) in this.as_cell().properties.iter().enumerate() {
            write!(
                buf,
                "{}: {}",
                key,
                match prop.to_string(rt) {
                    Ok(x) => x,
                    Err(e) => return Return::Error(e),
                }
            )
            .unwrap();
            if i != this.as_cell().properties.len() - 1 {
                write!(buf, ",").unwrap();
            }
        }
        write!(buf, " }}").unwrap();

        res = Value::from(rt.allocate_string(buf));
    } else {
        match this.to_string(rt) {
            Ok(x) => return Return::Return(Value::from(rt.allocate_string(x))),
            Err(e) => return Return::Error(e),
        }
    }
    Return::Return(res)
}

pub fn initialize(rt: &mut Runtime) {
    let f = native_fn!(rt, "toString", to_string);

    rt.object_prototype.put_named(f, "toString");
}
