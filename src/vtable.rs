use crate::builtins::*;
use crate::object::*;
use crate::value::Value;
use crate::*;
#[repr(C)]
pub struct VTable {
    pub trace_fn: Option<fn(Ref<Obj>, &mut dyn FnMut(Ref<Obj>))>,
    pub lookup_fn: Option<fn(&VM, Ref<Obj>, Value) -> WaffleResult>,
    pub index_fn: Option<fn(&VM, Ref<Obj>, usize) -> WaffleResult>,
    pub set_fn: Option<fn(&VM, Ref<Obj>, Value, Value) -> WaffleResult>,
    pub set_index_fn: Option<fn(&VM, Ref<Obj>, usize, Value) -> WaffleResult>,
    /// Calculate object size.
    pub calc_size_fn: Option<fn(Ref<Obj>) -> usize>,
    /// Object destructor, this should be used only by "external" objects that might contain
    /// pointers to non GC memory.
    pub destroy_fn: Option<fn(Ref<Obj>)>,
    /// Invoke object.
    pub apply_fn: Option<fn() -> WaffleResult>,
    pub parent: Option<&'static VTable>,
    pub instance_size: usize,
    pub element_size: usize,
}

impl VTable {
    pub fn is_array_ref(&self) -> bool {
        self as *const Self == &crate::builtins::ARRAY_VTBL as *const VTable
    }
}
