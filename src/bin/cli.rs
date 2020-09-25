use object::*;
use wafflelink::gc::*;
use wafflelink::isolate::Isolate;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

//#[global_allocator]
//static GLOBAL: std::alloc::System = std::alloc::System;

struct Foo {
    next: Option<Handle<Self>>,
}

impl GcObject for Foo {
    fn visit_references(&self, tracer: &mut Tracer) {
        self.next.visit_references(tracer);
    }
}
impl Drop for Foo {
    fn drop(&mut self) {}
}
fn main() {
    let isolate = Isolate::new();

    let val = isolate.new_local(42).to_heap();

    isolate.heap().minor();
}
