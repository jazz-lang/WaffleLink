use crate::runtime::*;
use value::*;
use crate::interpreter::Return;
use cell::*;
pub fn log(rt: &mut Runtime,_this: Value,args: &[Value]) -> Return {
    use chrono::prelude::Local;
    print!("{}: ",Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    for (i,arg) in args.iter().enumerate() {
        match arg.to_string(rt) {
            Ok(x) => print!("{}",x),
            Err(e) => return Return::Error(e),
        }
        if i != args.len() - 1 {
            print!(",");
        }
    }
    println!();
    Return::Return(Value::undefined())
}

macro_rules! native_fn {
    ($rt: expr,$name: expr,$func: expr) => {{
            let name = $rt.allocate_string($name);
            Value::from($rt.allocate_cell(
                Cell::new(
                    CellValue::Function(
                        Function::Native {
                            name: Value::from(name),
                            native: $func
                        }
                    ),Some($rt.function_prototype.to_heap())
                )
            )
            )
          }
    }

}

pub fn initialize(rt: &mut Runtime) {
    let func = native_fn!(rt,"log",log);
    rt.globals.insert("log".to_owned(),func);
}
