pub fn approximate_sp() -> *const usize {
    let x = 0usize;
    &x as *const usize
}
