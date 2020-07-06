use std::sync::atomic::AtomicU8;

macro_rules! offset_of {
    ($ty: ty, $field: ident) => {
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    };
}
pub(crate) static mut SAFEPOINT_PAGE: AtomicU8 = AtomicU8::new(0);
pub mod builtins;
pub mod bytecode;
pub mod function;
pub mod gc;
pub mod jit;
pub mod object;
pub mod pure_nan;
pub mod stack;
pub mod value;
pub mod vtable;
