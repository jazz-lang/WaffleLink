#![feature(thread_local)]
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

pub mod heap;
pub mod mutex;
pub mod safepoint;
pub mod signals;
pub mod threading;
pub mod utils;
