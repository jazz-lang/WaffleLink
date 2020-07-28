use crate::object::*;
use crate::value::*;
use crate::vtable::*;
use crate::*;
pub static ARRAY_VTBL: VTable = VTable {
    element_size: 8,
    instance_size: 0,
    parent: None,
    lookup_fn: Some(array_lookup),
    index_fn: None,
    calc_size_fn: Some(determine_array_size),
    apply_fn: None,
    destroy_fn: None,
    set_fn: Some(array_set),
    trace_fn: Some(trace_array),
    set_index_fn: None,
};

pub fn array_lookup(vm: &VM, this: Ref<Obj>, key: Value) -> WaffleResult {
    let this = this.cast::<Array>();
    if key == vm.length {
        return WaffleResult::okay(Value::new_int(this.len() as _));
    } else if key.is_number() {
        let idx = key.to_number().trunc() as usize;
        if idx < this.len() {
            return WaffleResult::okay(this.get_at(idx));
        } else {
            WaffleResult::okay(Value::undefined())
        }
    } else {
        WaffleResult::okay(Value::undefined())
    }
}

pub fn array_set(_: &VM, this: Ref<Obj>, key: Value, value: Value) -> WaffleResult {
    if !key.is_number() {
        return WaffleResult::okay(Value::new_bool(false));
    }
    let mut this = this.cast::<Array>();
    let idx = key.to_number().trunc() as usize;
    if idx < this.len() {
        this.set_at(idx, value);
        WaffleResult::okay(Value::new_bool(true))
    } else {
        WaffleResult::okay(Value::new_bool(false))
    }
}

pub fn trace_array(arr: Ref<Obj>, trace: &mut dyn FnMut(*const Ref<Obj>)) {
    let arr = arr.cast::<Array>();
    debug_assert!(arr.vtable as *const VTable == &ARRAY_VTBL as *const _);
    for i in 0..arr.len() {
        let item = arr.get_at(i);
        if item.is_cell() {
            trace(item.as_cell_ref());
        }
    }
}

fn determine_array_size(obj: Ref<Obj>) -> usize {
    let handle: Ref<Array> = obj.cast();

    let calc = Header::size() as usize
        + std::mem::size_of::<usize>()
        + std::mem::size_of::<usize>()
        + handle.vtable.element_size * handle.len() as usize;
    calc
}

pub static STRING_VTBL: VTable = VTable {
    element_size: std::mem::size_of::<WaffleString>(),
    instance_size: 0,
    parent: None,
    lookup_fn: None,
    index_fn: None,
    calc_size_fn: None,
    apply_fn: None,
    destroy_fn: None,
    set_fn: None,
    trace_fn: None,
    set_index_fn: None,
};
