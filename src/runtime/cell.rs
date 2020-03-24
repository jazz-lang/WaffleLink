use super::arc::*;
use super::value::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Cell {
    pub forward: crate::util::mem::Address,
}
impl Cell {
    #[inline(always)]
    pub fn new() -> Self {
        unsafe { std::hint::unreachable_unchecked() }
    }

    pub fn cast<T>(&self) -> *mut T {
        unsafe { std::mem::transmute(self) }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StringCell {
    pub header: Cell,
    pub length: usize,
    pub hash: usize,
    pub value: *mut u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alloc_string() {
        unsafe {
            let scell_layout = std::alloc::Layout::new::<StringCell>();
            let mem = std::alloc::alloc(
                std::alloc::Layout::from_size_align(scell_layout.size() + 14, scell_layout.align())
                    .unwrap(),
            ) as *mut Cell;
            let string = (&*mem).cast::<StringCell>();
            std::ptr::copy_nonoverlapping(
                b"Hello,World!".as_ptr(),
                (&mut *string).value,
                "Hello,World!".len(),
            );
            extern "C" {
                fn puts(raw: *mut u8);
            }
            puts((&*string).value);
        }
    }
}
