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
    fn visit_references(&self, _trace: &mut dyn FnMut(*const *mut GcBox<()>)) {
        self.next.visit_references(_trace);
    }
}
impl Drop for Foo {
    fn drop(&mut self) {}
}
use wafflelink::runtime::async_rt::*;
fn main() {
    let isolate = Isolate::new();
    isolate.run(|isolate| {
        isolate.spawn(async {
            println!("task #1: Hello,World!");
            yield_now().await;
            println!("task #1: end");
        });
        isolate.spawn(async {
            println!("task #2: hi!");
            yield_now().await;
            println!("task #2: end");
        });
    });
}
