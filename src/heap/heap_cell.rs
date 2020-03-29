#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct HeapCell;

pub const MIN_DISTANCE_FROM_CELLS_FROM_DIFFERENT_ORIGINS: usize =
    const_if!(std::mem::size_of::<usize>() == 8 => 304;288);

impl HeapCell {
    pub fn use_this(&self) {
        unsafe {
            asm!(
                "" : : "r"(self) : "memory" : "volatile"
            );
        }
    }

    pub fn is_zaped(&self) -> bool {
        unsafe { (*(self as *const Self as *const *const usize)).is_null() }
    }

    pub fn zap(&self) {
        unsafe {
            *(self as *const Self as *mut *mut usize) = std::ptr::null_mut();
        }
    }
}
