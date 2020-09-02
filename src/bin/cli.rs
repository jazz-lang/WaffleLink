use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use wafflelink::gc::*;

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
use wafflelink::timer::Timer;
fn main() {
    let mut timer = Timer::new(true);
    let mut heap = Heap::new(true, true);
    let mut root = heap.allocate(Foo { next: None });

    for _ in 0..1000 {
        let v2 = heap.allocate(Foo { next: None });

        let val = heap.allocate(Foo { next: None });

        heap.write_barrier(root.to_heap(), val.to_heap());
        root.next = Some(val.to_heap());
    }

    heap.collect_garbage();
    //drop(root);
    heap.collect_garbage_force(GcType::Major);
    heap.dump_summary(timer.stop());
    println!("{:p}", &root);
}
