use super::cell::*;
use super::process::*;
use super::scheduler::process_worker::ProcessWorker;
use super::state::*;
use super::value::*;
use super::*;
use crate::interpreter::context::*;
use crate::util::arc::Arc;

pub fn invoke_value(
    worker: &mut ProcessWorker,
    proc: &Arc<Process>,
    state: &RcState,
    value: Value,
    this: Value,
    args: Value,
) -> Result<ReturnValue, Value> {
    let cell = value.as_cell();
    let args = if args.is_cell() {
        match args.as_cell().get().value {
            CellValue::Array(ref array) => array.clone().to_vec(),
            _ => vec![],
        }
    } else {
        vec![]
    };
    match cell.get().value {
        CellValue::Function(ref func) => {
            if let Some(func) = func.native {
                return func(worker, state, proc, this, &args);
            } else {
                let mut ctx = Context::new();
                ctx.n = proc.context_ptr().n + 1;
                ctx.module = func.module.clone();
                ctx.code = func.code.clone();
                ctx.terminate_upon_return = true;
                ctx.stack = args;
                ctx.this = this;
                proc.push_context(ctx);
                let result = RUNTIME.run(worker, proc);
                return result.map(|x| ReturnValue::Value(x));
            }
        }
        _ => unimplemented!(),
    }
}

native_fn!(
    _worker,state,proc => bytecode this(..._args) {
        let this = this.as_cell();
        match this.get().value {
            CellValue::Function(ref func) => {
                let mut cell = Cell::with_prototype(CellValue::None,state.object_prototype.as_cell());
                /*cell.add_attribute(
                    Arc::new("constants".to_owned()),
                    Value::from(
                        Process::allocate(proc,
                            Cell::with_prototype(
                                CellValue::Array(Box::new(func.module.globals.clone())),
                                state.array_prototype.as_cell()
                            )
                        )
                        )
                    );*/

                let mut array = vec![];
                for bb in func.code.iter() {
                    let mut cell = Cell::with_prototype(CellValue::None,state.object_prototype.as_cell());
                    cell.add_attribute(Arc::new("index".to_owned()),Value::new_int(bb.index as _));
                    let mut ins_array = vec![];
                    for ins in bb.instructions.iter() {
                        let string = format!("{:?}",ins);
                        let mut split = string.split('(');
                        let name = split.next().unwrap();
                        let args = ins.args().iter().map(|x| if *x < std::u32::MAX as u64 {Value::new_int(*x as _)} else {Value::new_double(f64::from_bits(*x))}).collect();
                        let mut ins_cell = Cell::with_prototype(CellValue::None,state.object_prototype.as_cell());
                        ins_cell.add_attribute(Arc::new("name".to_owned()),Value::from(state.intern_string(name.to_owned())));
                        ins_cell.add_attribute(Arc::new("operands".to_owned()),Value::from(Process::allocate(proc,Cell::with_prototype(CellValue::Array(Box::new(args)),state.array_prototype.as_cell()))));
                        ins_array.push(Value::from(Process::allocate(proc,ins_cell)));
                    }

                    let array_ = Value::from(Process::allocate(proc,Cell::with_prototype(CellValue::Array(Box::new(ins_array)),state.array_prototype.as_cell())));
                    cell.add_attribute(Arc::new("instructions".to_owned()),array_);
                    array.push(Value::from(Process::allocate(proc,cell)));
                }

                let ret = Process::allocate(proc,Cell::with_prototype(CellValue::Array(Box::new(array)),state.array_prototype.as_cell()));
                Ok(ReturnValue::Value(Value::from(ret)))
            }
            _ => return Ok(ReturnValue::Value(Value::from(VTag::Null)))
        }
    }
);
native_fn!(
    _worker,state,proc => apply this (...args) {
        invoke_value(_worker,proc,state,this,args[0],args[1])
    }
);

pub fn initialize_function(state: &RcState) {
    let mut lock = state.static_variables.lock();
    let func = state.function_prototype.as_cell();
    func.add_attribute_without_barrier(
        &Arc::new("apply".to_owned()),
        Value::from(state.allocate_native_fn(apply, "apply", 2)),
    );
    func.add_attribute_without_barrier(
        &Arc::new("bytecode".to_owned()),
        Value::from(state.allocate_native_fn(bytecode, "bytecode", 0)),
    );
    lock.insert("Function".to_owned(), Value::from(func));
}
