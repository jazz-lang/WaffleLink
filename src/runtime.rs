use crate::*;
use bytecode::virtual_register::VirtualRegister;
use function::*;
use interpreter::callframe::*;
use object::*;
use table::*;
use value::*;
pub fn initialize() {
    use std::sync::Once;
    static RT_INIT: Once = Once::new();
    RT_INIT.call_once(|| {
        let _vm = crate::get_vm();
        register_global_fn(waffle_println, "print");
    });
}

pub fn register_global_fn(f: extern "C" fn(&mut CallFrame) -> WaffleResult, name: &str) {
    let vm = get_vm();
    let func = Function::new_native(&mut vm.heap, f, name);
    vm.globals.insert(name, Value::from(func.cast()));
    log!(
        "Add fn at {:p} with name {}, allocated at {:p}",
        f as *const u8,
        name,
        func.ptr
    );
}

pub extern "C" fn waffle_println(cf: &mut CallFrame) -> WaffleResult {
    let mut visited = HashSet::new();
    let mut buf = String::new();
    for i in 0..cf.passed_argc {
        let arg = cf.get_register(VirtualRegister::new_argument(i as _));
        write_val(&mut buf, arg, &mut visited).unwrap();
    }
    println!("{}", buf);
    WaffleResult::okay(Value::new_int(cf.passed_argc as _))
}

pub fn print_val(v: Value) {
    let mut visited = HashSet::new();
    let mut buf = String::new();
    write_val(&mut buf, v, &mut visited).unwrap();
    println!("{}", buf);
}
pub fn val_str(v: Value) -> String {
    let mut visited = HashSet::new();
    let mut buf = String::new();
    write_val(&mut buf, v, &mut visited).unwrap();
    buf
}
use std::collections::HashSet;
pub fn write_val(
    buffer: &mut dyn std::fmt::Write,
    val: Value,
    visited: &mut HashSet<u64>,
) -> std::fmt::Result {
    if val.is_number() {
        write!(buffer, "{}", val.to_number())?;
    } else if val.is_boolean() {
        write!(buffer, "{}", val.to_boolean())?;
    } else if val.is_undefined() {
        write!(buffer, "undefined")?;
    } else if val.is_null() {
        write!(buffer, "null")?;
    } else if val.is_cell() {
        let c = val.as_cell();
        let ix = c.ptr.as_ptr() as usize as u64;
        if visited.contains(&ix) && (c.is_array_ref() || c.is_robj()) {
            write!(buffer, "...")?;
        } else {
            visited.insert(ix);
            if c.is_string() {
                write!(buffer, "{}", c.cast::<WaffleString>().str())?;
            } else if c.is_array_ref() {
                let arr = c.cast::<Array>();
                write!(buffer, "[")?;
                for i in 0..arr.len() {
                    write_val(buffer, arr.get_at(i), visited)?;
                    if i != arr.len() - 1 {
                        write!(buffer, ",")?;
                    }
                }
                write!(buffer, "]")?;
            } else if c.is_robj() {
                let obj = c.cast::<RegularObj>();
                write!(buffer, "{{ ")?;
                match &obj.table.table {
                    TableEnum::Fast(cls, fields) => {
                        if let Some(d) = &cls.descriptors {
                            for (key, desc) in d.iter() {
                                if let Descriptor::Property(ix) = desc {
                                    write!(buffer, "{} => ", key.str())?;
                                    write_val(buffer, fields[*ix as usize], visited)?;
                                }
                            }
                        }
                    }
                    TableEnum::Slow(_, props) => {
                        for (key, val) in props.iter() {
                            write!(buffer, "{} => ", key.str())?;
                            write_val(buffer, *val, visited)?;
                        }
                    }
                }
                write!(buffer, "}}")?;
            } else if c.is_function() {
                let f = c.cast::<Function>();
                write!(
                    buffer,
                    "function {}(...){{...}} at {:p}",
                    f.name.str(),
                    f.ptr
                )?;
            } else {
                write!(buffer, "[object at {:p}]", ix as *const u8)?;
            }
        }
    }
    Ok(())
}
