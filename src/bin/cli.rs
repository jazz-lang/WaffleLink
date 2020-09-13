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
    let begin = 0;
    println!("{}", core::mem::size_of::<GcBox<Foo>>());
    let mut gc = TGC::new(&begin, Some(8), true);
    let mut scope = gc.new_local_scope();

    let mut roots = scope.allocate(vec![]);
    for _ in 0..1000 {
        roots.push(gc.allocate_no_root(2));
    }

    let mut roots2 = scope.allocate(vec![]);
    for _ in 0..1000 {
        roots2.push(gc.allocate_no_root(2));
    }
    drop(roots2);
    let gc_start = std::time::Instant::now();
    gc.collect_garbage(&0);

    let end = gc_start.elapsed();
    println!("GC done in {}ns {}ms", end.as_nanos(), end.as_millis());
    drop(roots);

    let gc_start = std::time::Instant::now();
    gc.force_major_gc();
    let end = gc_start.elapsed();
    println!("GC done in {}ns", end.as_nanos());
    unsafe {
        libmimalloc_sys::mi_collect(true);
    }
}
