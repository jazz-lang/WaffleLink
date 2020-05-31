use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
static B: AtomicBool = AtomicBool::new(false);
fn main() {
    let t = std::time::Instant::now();
    B.store(true, Ordering::Relaxed);
    let x = B.load(Ordering::Relaxed);

    let e = t.elapsed();
    println!("{} {}", x, e.as_nanos());
}
