use super::value::*;
pub const STACK_CAPACITY: usize = 16 * 1024;
pub const STACK_BYTES: usize = STACK_CAPACITY * std::mem::size_of::<u64>();
pub const COMMIT_SIZE: usize = 4 * 1024;

pub const FRAME_SIZE: i32 = roundup(std::mem::size_of::<Frame>(), std::mem::size_of::<Value>())
    as i32
    / std::mem::size_of::<Value>() as i32;
pub const CONSTANT_OFFSET: usize = STACK_CAPACITY;

pub const fn roundup(x: usize, y: usize) -> usize {
    ((x) + (y - 1)) & !(y - 1)
}
pub fn callee_offset() -> isize {
    -(FRAME_SIZE as isize)
        + offset_of!(Frame, callee) as isize / std::mem::size_of::<Value>() as isize
}
pub struct Frame {
    callee: usize,
}
