#![feature(thread_local, min_specialization, const_maybe_uninit_assume_init)]
#[macro_use]
extern crate mopa;
pub trait IntoAtomic {
    type Output;
    fn into_atomic(&self) -> &'static Self::Output;
}

impl IntoAtomic for usize {
    type Output = std::sync::atomic::AtomicUsize;
    fn into_atomic(&self) -> &'static Self::Output {
        unsafe { std::mem::transmute(self) }
    }
}

impl IntoAtomic for u8 {
    type Output = std::sync::atomic::AtomicU8;
    fn into_atomic(&self) -> &'static Self::Output {
        unsafe { std::mem::transmute(self) }
    }
}

impl IntoAtomic for u64 {
    type Output = std::sync::atomic::AtomicU64;
    fn into_atomic(&self) -> &'static Self::Output {
        unsafe { std::mem::transmute(self) }
    }
}

#[macro_export]
macro_rules! as_atomic {
    ($value: expr;$t: ident) => {
        unsafe { std::mem::transmute::<_, &'_ std::sync::atomic::$t>($value as *const _) }
    };
}

/// Creates struct with layout suitable for big or little endian machine
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

pub mod callframe;
pub mod heap;
pub mod mutex;
pub mod object;
pub mod safepoint;
pub mod signals;
pub mod threading;
pub mod utils;
