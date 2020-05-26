use super::{cell::*, transition_map::*, value::*, vtable::*, *};
use crate::common::*;
use crate::gc::*;
pub static OBJECT_VTBL: VTable = VTable {
    get: Some(get),
    set: Some(set),
    get_class: Some(get_class),
    get_table: None,
    parent: None,
};

pub fn get(this: Handle<Cell>, key: Value) -> Result<Option<Value>, Value> {
    let str = key.to_string()?;

    let object = this.get().to::<Object>();
    Ok(object.table.load(&str))
}

pub fn set(this: Handle<Cell>, key: Value, val: Value) -> bool {
    match key.to_string() {
        Ok(str) => {
            let object = this.get().to::<Object>();
            object.table.set(get_rt().heap.allocate(str), val);
        }
        _ => return false,
    };
    true
}

pub fn get_class(this: Handle<Cell>) -> Option<Handle<Class>> {
    Some(this.get().to::<Object>().table.class())
}

pub fn get_table(this: Handle<Cell>) -> DerefPointer<Table> {
    DerefPointer::new(&this.get().to::<Object>().table)
}
