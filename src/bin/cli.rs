use object::*;
use wafflelink::gc::*;
use wafflelink::runtime::array::Array;
use wafflelink::values::*;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

//#[global_allocator]
//static GLOBAL: std::alloc::System = std::alloc::System;

struct Foo {
    next: Option<Handle<Self>>,
}

impl GcObject for Foo {
    fn visit_references(&self, _trace: &mut dyn FnMut(*const GcBox<()>)) {
        self.next.visit_references(_trace);
    }
}
impl Drop for Foo {
    fn drop(&mut self) {}
}

fn main() {
    let mut heap = Heap::lazysweep();
    let mut scope = heap.new_local_scope();

    let arr = Array::new_local(&mut scope, Value::undefined(), 16);

    arr.for_each(|x| {
        assert!(x.is_undefined());
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 128);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    })
}
