use crate::object::*;
use crate::vtable::*;

pub static BIGINT_VTBL: VTable = VTable {
    element_size: 0,
    instance_size: std::mem::size_of::<BigIntObject>(),
    parent: None,
    lookup_fn: None,
    index_fn: None,
    calc_size_fn: None,
    apply_fn: None,
    destroy_fn: Some(destroy_bigint),
    set_fn: None,
    set_index_fn: None,
    trace_fn: None,
};

fn destroy_bigint(x: Ref<Obj>) {
    println!("ded");
    let this = x.cast::<BigIntObject>();
    unsafe {
        std::ptr::drop_in_place(this.ptr as *mut BigIntObject);
    }
}

#[repr(C)]
pub struct BigIntObject {
    pub header: Header,
    pub bigint: num_bigint::BigInt,
}

use crate::heap::Heap;

impl BigIntObject {
    pub fn new(heap: &mut Heap, sign: num_bigint::Sign, digits: Vec<u32>) -> Ref<Self> {
        let mem = heap.allocate(std::mem::size_of::<Self>());
        let mut this: Ref<BigIntObject> = Ref {
            ptr: mem.to_mut_ptr(),
        };
        this.header.init_vtbl(&BIGINT_VTBL);
        this.bigint = num_bigint::BigInt::new(sign, digits);
        this
    }
}
