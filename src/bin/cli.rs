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
use wafflelink::utils::fast_bitvec::*;

fn main() {
    let mut bv = FastBitVector::new();
    bv.resize(64);
    bv.atomic_set_and_check(34, true);
    let mut bv1 = FastBitVector::new();
    bv1.resize(64);
    bv1.set_at(30, true);
    bv |= bv1;
    println!("{:?}", bv);
    bv.for_each_clear_bit(|bit| {
        println!("{}", bit);
    });
}
