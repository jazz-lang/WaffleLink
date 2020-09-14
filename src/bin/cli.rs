use object::*;
use wafflelink::gc::*;
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
    let mut local = heap.new_local_scope();
    for _ in 0..block::PAYLOAD_SIZE / 32 - 4 {
        let __: Local<'_, i32> = local.allocate(42);
    }

    heap.minor();
    let __ = local.allocate(3);
}
