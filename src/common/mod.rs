pub mod attributes;
pub mod freelist;
pub mod map;
pub mod mem;
pub mod ptr;
pub mod space;
pub mod symbol;

pub fn next_capacity(cap: usize) -> usize {
    if cap == 0 {
        return 0;
    } else if cap < 8 {
        return 8;
    } else {
        clp2(cap)
    }
}

pub fn flp2(mut x: usize) -> usize {
    x = x | (x >> 1);
    x = x | (x >> 2);
    x = x | (x >> 4);
    x = x | (x >> 8);
    x = x | (x >> 16);
    return x - (x >> 1);
}

pub fn clp2(x: usize) -> usize {
    if x == 0 || x > 0x80000000 {
        0
    } else {
        flp2(x)
    }
}
