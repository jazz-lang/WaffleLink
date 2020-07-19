use crate::bytecode::*;
use crate::heap::*;
use crate::object::*;
use crate::vtable::*;
#[repr(C)]
pub struct Function {
    header: Header,
    pub(crate) vtable: &'static VTable,
    pub(crate) code_block: Ref<CodeBlock>,
    pub native: bool,
    pub native_code: usize,
}

impl Function {
    pub fn new_native(
        heap: &mut Heap,
        fptr: extern "C" fn(&mut crate::interpreter::callframe::CallFrame) -> crate::WaffleResult,
    ) -> Ref<Self> {
        let mem = heap.allocate(std::mem::size_of::<Self>());
        unsafe {
            mem.to_mut_ptr::<Self>().write(Self {
                header: Header::new(),
                vtable: &FUNCTION_VTBL,
                code_block: Ref {
                    ptr: std::ptr::null(),
                },
                native: true,
                native_code: fptr as _,
            });
        }
        let cell = Ref {
            ptr: mem.to_ptr::<Self>(),
        };
        return cell;
    }
}

pub static FUNCTION_VTBL: VTable = VTable {
    element_size: 0,
    instance_size: std::mem::size_of::<Function>(),
    parent: None,
    lookup_fn: None,
    index_fn: None,
    calc_size_fn: None,
    apply_fn: None,
    destroy_fn: None,
    trace_fn: None,
    set_fn: None,
    set_index_fn: None,
};
