use criterion::*;
use std::sync::atomic::*;
pub fn criterion_benchmark(c: &mut Criterion) {
    static X: AtomicUsize = AtomicUsize::new(0);
    #[derive(Debug)]
    pub struct Node {
        x: i32,
        next: Option<Box<Self>>,
    }
    pub struct List {
        head: Option<Box<Node>>,
    }
    impl List {
        pub fn new() -> Self {
            Self { head: None }
        }

        pub fn push(&mut self, x: i32) {
            let node = Box::new(Node {
                x,
                next: self.head.take(),
            });
            self.head = Some(node);
        }

        pub fn pop(&mut self) -> Option<Box<Node>> {
            let mut head = self.head.take();
            if head.is_none() {
                return None;
            }
            self.head = head.as_mut().unwrap().next.take();
            head
        }
    }

    c.bench_function("fetch_add", |b| {
        b.iter(|| {
            let mut l = List::new();
            l.push(2);
            criterion::black_box(l.pop());
            let _ = X.fetch_add(1, Ordering::Relaxed);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
