use crate::object::*;
use crate::value::*;
use crate::vtable::*;
pub static ARRAY_VTBL: VTable = VTable {
    element_size: 8,
    instance_size: 0,
    parent: None,
    lookup_fn: None,
    index_fn: None,
    calc_size_fn: Some(determine_array_size),
    apply_fn: None,
    destroy_fn: None,
    set_fn: None,
    trace_fn: Some(trace_array),
    set_index_fn: None,
};
pub fn trace_array(arr: Ref<Obj>, trace: &mut dyn FnMut(Ref<Obj>)) {
    let arr = arr.cast::<Array>();
    for i in 0..arr.len() {
        let item = arr.get_at(i);
        if item.is_cell() {
            trace(item.as_cell());
        }
    }
}

fn determine_array_size(obj: Ref<Obj>) -> usize {
    let handle: Ref<Array> = Ref {
        ptr: obj.ptr as *const Obj as *const Array,
    };

    let calc = Header::size() as usize
        + std::mem::size_of::<usize>()
        + std::mem::size_of::<usize>()
        + handle.vtable.element_size * handle.len() as usize;
    calc
}

pub static STRING_VTBL: VTable = VTable {
    element_size: std::mem::size_of::<char>(),
    instance_size: 0,
    parent: None,
    lookup_fn: None,
    index_fn: None,
    calc_size_fn: Some(determine_array_size),
    apply_fn: None,
    destroy_fn: None,
    set_fn: None,
    trace_fn: Some(trace_array),
    set_index_fn: None,
};
