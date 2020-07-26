use crate::bytecode::*;
use crate::heap::*;
use crate::object::*;
use crate::vtable::*;
use crate::*;
#[repr(C)]
pub struct Function {
    header: Header,
    pub(crate) vtable: &'static VTable,
    pub(crate) code_block: Option<Ref<CodeBlock>>,
    pub native: bool,
    pub native_code: usize,
    pub env: Option<Ref<Array>>,
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
                code_block: None,
                env: None,
                native: true,
                native_code: fptr as _,
            });
        }
        Ref {
            ptr: std::ptr::NonNull::new(mem.to_mut_ptr()).unwrap(),
        }
    }

    pub fn new(heap: &mut Heap, cb: Ref<CodeBlock>) -> Ref<Self> {
        let mem = heap.allocate(std::mem::size_of::<Self>());
        unsafe {
            mem.to_mut_ptr::<Self>().write(Self {
                header: Header::new(),
                vtable: &FUNCTION_VTBL,
                code_block: Some(cb),
                env: None,
                native: false,
                native_code: 0,
            });
        }
        Ref {
            ptr: std::ptr::NonNull::new(mem.to_mut_ptr()).unwrap(),
        }
    }

    pub fn execute(&self, this: value::Value, args: &[value::Value]) -> WaffleResult {
        use interpreter::callframe::*;
        let regc = if let Some(cb) = self.code_block {
            cb.num_vars
        } else {
            0
        };
        let callee = Ref {
            ptr: std::ptr::NonNull::new(self as *const Self as *mut Self).unwrap(),
        };
        let callee = value::Value::from(callee.cast());
        let vm = get_vm();
        let mut cf = CallFrame::new(args, regc);
        cf.this = this;
        cf.callee = callee;
        cf.passed_argc = args.len() as _;
        cf.code_block = self.code_block;
        if self.native {
            let f: extern "C" fn(
                &mut crate::interpreter::callframe::CallFrame,
            ) -> crate::WaffleResult = unsafe { std::mem::transmute(self.native_code) };
            vm.call_stack.push(cf);
            let res = f(vm.call_stack.last_mut().unwrap());
            vm.call_stack.pop();
            res
        } else {
            if let Some((fun, _argc, vars, cb)) =
                jit::operations::get_executable_address_for(callee)
            {
                vm.call_stack.push(cf);
                let result = fun(&mut vm.call_stack.last_mut().unwrap());
                vm.call_stack.pop().unwrap();
                return result;
            } else {
                todo!()
            }
        }
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
