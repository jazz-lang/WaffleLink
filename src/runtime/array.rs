use super::{cell::*, transition_map::*, value::*, vtable::*, *};
use crate::common::*;
use crate::gc::*;

pub static ARRAY_VTBL: VTable = VTable {
    get: Some(get),
    set: Some(set),
    get_class: Some(get_class),
    get_table: Some(get_table),
    parent: None,
};

pub fn get(this: Handle<Cell>, key: Value) -> Result<Option<Value>, Value> {
    let this = this.get().to::<Array>();
    if key.is_number() {
        let idx = key.to_number().floor() as usize;
        if idx < this.array.len() {
            return Ok(Some(this.array[idx]));
        }
    }
    let key = key.to_string()?;
    let value = this.table.load(&key);

    Ok(value)
}

pub fn set(this: Handle<Cell>, key: Value, val: Value) -> bool {
    let this = this.get().to::<Array>();
    if key.is_number() {
        let idx = key.to_number().floor() as usize;
        if idx < this.array.len() {
            this.array[idx] = val;
            return true;
        }
    }
    let key = match key.to_string() {
        Ok(s) => s,
        _ => return false,
    };
    this.table.set(get_rt().heap.allocate(key), val);
    true
}

pub fn get_class(this: Handle<Cell>) -> Option<Handle<Class>> {
    Some(this.get().to::<Array>().table.class())
}

pub fn get_table(this: Handle<Cell>) -> DerefPointer<Table> {
    DerefPointer::new(&this.get().to::<Array>().table)
}
