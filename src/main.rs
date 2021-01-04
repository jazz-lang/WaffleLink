#[macro_use]
extern crate wafflelink;

new_const_bitmap!(MyBitmap, 16, 32 * 1024);

fn main() {
    println!(
        "{}",
        wafflelink::heap::align_usize(core::mem::size_of::<MyBitmap>() * 2 + 8 + 2, 16)
    );
    let mut heap = Box::new([0u8; 16 * 1024]);
    let raw = &heap[0] as *const u8 as *mut u8;
    let mut bitmap = MyBitmap::new(raw);
    let mut live = MyBitmap::new(raw);
    bitmap.set(&heap[16 * 32] as *const u8 as *mut u8 as _);
    live.set(&heap[16 * 3] as *const u8 as usize);
    live.set(&heap[16 * 36] as *const u8 as usize);
    live.set(&heap[16 * 32] as *const u8 as usize);
    live.set(&heap[16 * 38] as *const u8 as *mut u8 as _);
    unsafe {
        MyBitmap::sweep_walk(
            &live,
            &bitmap,
            raw as _,
            heap.last().unwrap() as *const u8 as usize,
            |cnt, objects| {
                println!("{:p}", &heap[16 * 36]);
                let objects = objects as *mut *mut u8;
                println!(
                    "free {}:{:p}->{:p}",
                    cnt,
                    objects.read(),
                    objects.offset(cnt as isize - 1).read()
                );
            },
        );
    }

    drop(heap);
}
