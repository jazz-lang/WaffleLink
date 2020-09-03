use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use wafflelink::heap::segregated_storage::*;
fn main() {
    unsafe {
        println!("size classes \n{:#?}", *SIZE_CLASSES);
        let mut block = &mut *Block::boxed(index_to_size_class(5));

        let p1 = block.allocate();
        let p2 = block.allocate();
        let p3 = block.allocate();
        block.mark(p1);
        block.mark(p3);
        println!(
            "{} {} {}",
            block.is_marked(p1),
            block.is_marked(p2),
            block.is_marked(p3)
        );
    }
}
