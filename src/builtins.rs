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
    trace_fn: None,
    set_index_fn: None,
};

fn determine_array_size(obj: Ref<Obj>) -> usize {
    let handle: Ref<Array> = Ref {
        ptr: obj.ptr as *const Obj as *const Array,
    };

    let calc = Header::size() as usize
        + std::mem::size_of::<usize>()
        + std::mem::size_of::<Value>() * handle.len() as usize;
    calc
}

#[repr(C)]
pub struct WResult {
    ok: bool,
    value: Value,
}

impl WResult {
    #[inline(always)]
    pub fn to_result(&self) -> Result<Value, Value> {
        if self.ok {
            Ok(self.value)
        } else {
            Err(self.value)
        }
    }
    #[allow(non_snake_case)]
    #[inline]
    pub const fn Ok(value: Value) -> Self {
        Self { ok: true, value }
    }
    #[allow(non_snake_case)]
    #[inline]
    pub const fn Err(value: Value) -> Self {
        Self { ok: false, value }
    }
}
