use std::collections::HashSet;
use wafflelink::gc::*;

fn main() {
    let mut hash = HashSet::with_capacity(4);
    let start = std::time::Instant::now();
    hash.insert(&hash as *const _ as usize);
    println!("{}", start.elapsed().as_nanos());
}
