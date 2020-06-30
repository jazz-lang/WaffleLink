pub mod callframe;

#[repr(C)]
pub struct StackInfo {
    // pointer to previous info
    pub last: *const StackInfo,

    // frame pointer of native stub
    pub fp: usize,

    // some program counter into native stub
    pub pc: usize,
}

impl StackInfo {
    pub const fn new() -> Self {
        Self {
            last: std::ptr::null(),
            fp: 0,
            pc: 0,
        }
    }
}
