use block::*;
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

struct VeryLarge([u8;10*1024]);

impl GcObject for VeryLarge {}

fn main() {
    let mut heap = Heap::lazysweep();
    let mut local = heap.new_local_scope();
    println!("{}",block::PAYLOAD_SIZE/32);
    for _ in 0..block::PAYLOAD_SIZE/32 - 4 {
        let xx = local.allocate(42);
    }
    //drop(xx);
    heap.minor();
    let yy = local.allocate(3);   
}
