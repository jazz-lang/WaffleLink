#[macro_use]
extern crate wafflelink;
use object::*;
use wafflelink::gc::*;
use wafflelink::isolate::Isolate;
use wafflelink::prelude::*;

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

    let mut map = Map::new(
        &isolate,
        wafflelink::runtime::map::compute_hash_default,
        wafflelink::runtime::map::iseq_default,
        8,
    );
    let m1 = (*map).clone();
}
