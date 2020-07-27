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
    pub name: Ref<WaffleString>,
    pub prototype: value::Value,
}

fn lookup_fn(vm: &VM,this: Ref<Obj>,key: value::Value) -> WaffleResult {
    if key == vm.constructor {
        return WaffleResult::okay(value::Value::from(this))
    } else if key == vm.prototype {
        return WaffleResult::okay(value::Value::from(this.cast::<Function>().prototype))
    } else {
        WaffleResult::okay(value::Value::undefined())
    }
}

impl Function {
    pub fn new_native(
        heap: &mut Heap,
        fptr: extern "C" fn(&mut crate::interpreter::callframe::CallFrame) -> crate::WaffleResult,
        name: &str,
    ) -> Ref<Self> {
        let mem = heap.allocate(std::mem::size_of::<Self>());
        unsafe {
            mem.to_mut_ptr::<Self>().write(Self {
                header: Header::new(),
                vtable: &FUNCTION_VTBL,
                code_block: None,
                env: None,
                native: true,prototype: value::Value::undefined(),
                name: WaffleString::new(heap, name),
                native_code: fptr as _,
            });
        }
        Ref {
            ptr: std::ptr::NonNull::new(mem.to_mut_ptr()).unwrap(),
        }
    }

    pub fn new(heap: &mut Heap, cb: Ref<CodeBlock>, name: &str) -> Ref<Self> {
        let mem = heap.allocate(std::mem::size_of::<Self>());
        unsafe {
            mem.to_mut_ptr::<Self>().write(Self {
                header: Header::new(),
                vtable: &FUNCTION_VTBL,
                code_block: Some(cb),
                env: None,
                native: false,
                native_code: 0,
                prototype: value::Value::from(RegularObj::new(heap,value::Value::undefined(),None).cast()),
                name: WaffleString::new(heap, name),
            });
        }
        Ref {
            ptr: std::ptr::NonNull::new(mem.to_mut_ptr()).unwrap(),
        }
    }

    pub fn execute(&self, this: value::Value, args: &[value::Value]) -> WaffleResult {
        
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
        let cf = vm.push_frame(args, regc);
        cf.this = this;
        cf.callee = callee;
        cf.passed_argc = args.len() as _;
        cf.code_block = self.code_block;
        if self.native {
            let f: extern "C" fn(
                &mut crate::interpreter::callframe::CallFrame,
            ) -> crate::WaffleResult = unsafe { std::mem::transmute(self.native_code) };
            
            let res = f(cf);
            vm.pop_frame();
            res
        } else {
            if let Some((fun, _argc, _vars, _cb)) =
                jit::operations::get_executable_address_for(callee)
            {
                //vm.call_stack.push(cf);
                let result = fun(cf);
                vm.pop_frame();
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
    lookup_fn: Some(lookup_fn),
    index_fn: None,
    calc_size_fn: None,
    apply_fn: None,
    destroy_fn: None,
    trace_fn: None,
    set_fn: None,
    set_index_fn: None,
};
