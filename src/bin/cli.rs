use object::*;
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
use wafflelink::utils::fast_bitvec::*;

fn main() {
    let begin = 0;
    let mut gc = TGC::new(&begin, Some(3), true);

    let mut roots: Root<Vec<Handle<i32>>> = gc.allocate(vec![]);
    for _ in 0..1000 {
        roots.push(gc.allocate(0).to_heap());
    }
    let mut roots2: Root<Vec<Handle<i32>>> = gc.allocate(vec![]);
    for _ in 0..1000 {
        roots2.push(gc.allocate(2).to_heap());
    }
    let gc_start = std::time::Instant::now();
    gc.collect_garbage(&0);
    let end = gc_start.elapsed();
    println!("GC done in {}ns", end.as_nanos());
    drop(roots);

    gc.collect_garbage(&1);
}
