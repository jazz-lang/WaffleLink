use super::cell::*;
use super::process::*;
use super::scheduler::process_worker::ProcessWorker;
use super::state::*;
use super::value::*;
use crate::util::arc::Arc;

fn isdigit(c: char) -> bool {
    ('0' as u32) <= (c as u32) && (c as u32) <= ('9' as u32)
}

fn isalpha(c: char) -> bool {
    let c = c as u32;
    ('A' as u32) <= c && c <= ('Z' as u32) || ('a' as u32) <= c && c <= ('z' as u32)
}

fn isspace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n'
}

fn islower(c: char) -> bool {
    ('a' as u32) <= (c as u32) && (c as u32) <= ('z' as u32)
}

fn isupper(c: char) -> bool {
    ('A' as u32) <= (c as u32) && (c as u32) <= ('Z' as u32)
}

native_fn!(
    _worker,state,proc => ctor this (arg)
        if this.is_cell() {
            this.as_cell().get_mut().value = CellValue::String(Arc::new(arg.to_string()));
            Ok(ReturnValue::Value(this))
        } else {
            return Ok(ReturnValue::Value(Value::from(Process::allocate_string(proc,state,&arg.to_string()))))
        }

);

native_fn!(
    _worker,state,proc => char_at this (index)
        Ok(ReturnValue::Value(this.to_string().chars().nth(index.to_number().floor() as usize).map(|val| Value::from(Process::allocate_string(proc,state,&(val.to_string())))).unwrap_or(Value::from(VTag::Undefined))))

);
native_fn!(
    _worker,state,proc => split this(...args) {
        let string = this.to_string();
        if args.is_empty() {
            let array = string.split("").map(|x| Value::from(Process::allocate_string(proc,state,x))).collect::<Vec<_>>();
            let array = Process::allocate(proc,Cell::with_prototype(CellValue::Array(Box::new(array)),state.array_prototype.as_cell()));
            return Ok(
                ReturnValue::Value(
                    Value::from(array)
                )
            )
        } else {
            let array = string.split(&args[0].to_string()).map(|x| Value::from(Process::allocate_string(proc,state,x))).collect::<Vec<_>>();
            let array = Process::allocate(proc,Cell::with_prototype(CellValue::Array(Box::new(array)),state.array_prototype.as_cell()));
            return Ok(
                ReturnValue::Value(
                    Value::from(array)
                )
            )
        }

    }
);

native_fn!(_worker,_state,_proc => string_is_digit this (...args) {
    let base = match args.len() {
        0 => 10u32,
        1 => args[0].to_number().floor() as u32,
        _n => 10u32,
    };
    for c in this.to_string().chars() {
        if !c.is_digit(base) {
            return Ok(ReturnValue::Value(Value::from(false)))
        }
    }
    return Ok(ReturnValue::Value(Value::from(true)))
});
native_fn!(_worker,_state,_proc => string_is_alpha this (..._args) {
    for c in this.to_string().chars() {
        if !isalpha(c) {return Ok(ReturnValue::Value(Value::from(false)))}
    }
    return Ok(ReturnValue::Value(Value::from(true)))
});

native_fn!(_worker,_state,_proc => string_is_space this (..._args) {
    for c in this.to_string().chars() {
        if !isspace(c) {
            return Ok(ReturnValue::Value(Value::from(false)))
        }
    }
    return Ok(ReturnValue::Value(Value::from(true)))
});

native_fn!(_worker,state,proc => string_chars this (..._args) {
    let array = this.to_string().chars().map(|ch| Value::from(Process::allocate_string(proc,state,&ch.to_string()))).collect::<Vec<_>>();
    return Ok(ReturnValue::Value(Value::from(Process::allocate(proc,Cell::with_prototype(CellValue::Array(Box::new(array)),state.array_prototype.as_cell())))));
});

native_fn!(_worker,_state,_proc => string_length this(..._args) {
    return Ok(ReturnValue::Value(Value::new_int(this.to_string().len() as _)));
});

native_fn!(_worker,state,proc => replace this(...args) {
    if args.is_empty() || args.len() < 2 {
        return Ok(ReturnValue::Value(this));
    } else {
        let this = this.to_string();
        let from = args[0].to_string();
        let to = args[1].to_string();
        let replaced = this.replace(&from,&to);
        Ok(ReturnValue::Value(Value::from(Process::allocate_string(proc, state, &replaced))))
    }
});

pub fn initialize_string(state: &RcState) {
    let mut lock = state.static_variables.lock();
    let cell = state.string_prototype.as_cell();
    cell.add_attribute_without_barrier(
        &Arc::new("split".to_owned()),
        Value::from(state.allocate_native_fn(split, "split", -1)),
    );
    cell.add_attribute_without_barrier(
        &Arc::new("charAt".to_owned()),
        Value::from(state.allocate_native_fn(char_at, "charAt", 1)),
    );
    cell.add_attribute_without_barrier(
        &Arc::new("constructor".to_owned()),
        Value::from(state.allocate_native_fn(ctor, "constructor", 1)),
    );
    cell.add_attribute_without_barrier(
        &Arc::new("isDigit".to_owned()),
        Value::from(state.allocate_native_fn(string_is_digit, "isDigit", 0)),
    );
    cell.add_attribute_without_barrier(
        &Arc::new("isSpace".to_owned()),
        Value::from(state.allocate_native_fn(string_is_space, "isSpace", 0)),
    );
    cell.add_attribute_without_barrier(
        &Arc::new("isAlpha".to_owned()),
        Value::from(state.allocate_native_fn(string_is_alpha, "isAlpha", 0)),
    );
    cell.add_attribute_without_barrier(
        &Arc::new("chars".to_owned()),
        Value::from(state.allocate_native_fn(string_chars, "chars", 0)),
    );
    cell.add_attribute_without_barrier(
        &Arc::new("length".to_owned()),
        Value::from(state.allocate_native_fn(string_length, "length", 0)),
    );
    cell.add_attribute_without_barrier(
        &Arc::new("replace".to_owned()),
        Value::from(state.allocate_native_fn(replace, "replace", -1)),
    );
    lock.insert("String".to_owned(), Value::from(cell));
}
