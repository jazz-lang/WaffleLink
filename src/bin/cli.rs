#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;
#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use object::*;
use wafflelink::heap::*;

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
    let mut s = 0;
    let mut heap = TGC::new(&s);
    let mut root = heap.allocate(Foo { next: None });

    for _ in 0..1000 {
        let v2 = heap.allocate(Foo { next: None });

        let val = heap.allocate(Foo { next: None });

        //heap.write_barrier(root.to_heap(), val.to_heap());
        root.next = Some(val.to_heap());
    }
    let end = 0;
    heap.collect_garbage(&end);
    drop(root);
    heap.collect_garbage(&end);
    //heap.collect_garbage_force(GcType::Major);

    //heap.dump_summary(timer.stop());
}
