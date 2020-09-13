use block::*;
use object::*;
use wafflelink::gc::*;
//#[global_allocator]
//static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;

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
    unsafe {
        let block = Block::new(32);
        println!("{}", block.atoms_per_cell());
    }
}
