//#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_mut, dead_code, unused_variables)]

#[macro_export]
macro_rules! lohi_struct {
    (struct $name : ident {
        $field1: ident : $t: ty,
        $field2: ident : $t2: ty,
    }) => {
        #[derive(Copy, Clone, PartialEq, Eq)]
        #[repr(C)]
        #[cfg(target_endian = "big")]
        pub struct $name {
            pub $field2: $t2,
            pub $field1: $t,
        }
        #[derive(Copy, Clone, PartialEq, Eq)]
        #[repr(C)]
        #[cfg(target_endian = "little")]
        pub struct $name {
            pub $field1: $t,
            pub $field2: $t,
        }
    };
}
#[macro_export]
macro_rules! unused {
    ($($var: ident),*) => {
        $(
            let _ = $var;
        )*
    };
}

#[macro_export]
macro_rules! const_assert {
    ($x:expr $(,)?) => {
        #[allow(unknown_lints, clippy::eq_op)]
        const _: [(); 0 - !{
            const ASSERT: bool = $x;
            ASSERT
        } as usize] = [];
    };
}
pub mod gc;
pub mod heap;
pub mod timer;
pub mod utils;
pub mod values;
pub mod vm;

pub struct VM {
    heap: std::cell::UnsafeCell<heap::Heap>,
}

pub static VM_INSTANCE: once_cell::sync::Lazy<VM> = once_cell::sync::Lazy::new(|| VM {
    heap: std::cell::UnsafeCell::new(heap::Heap::new()),
});

impl VM {
    pub fn heap(&self) -> &mut heap::Heap {
        unsafe { &mut *self.heap.get() }
    }
}

unsafe impl Send for VM {}
unsafe impl Sync for VM {}

pub fn vm() -> &'static VM {
    &*VM_INSTANCE
}
