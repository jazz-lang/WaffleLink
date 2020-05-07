extern crate cgc_single_threaded as cgc;

#[macro_export]
macro_rules! offset_of {
    ($ty: ty, $field: ident) => {
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    };
}

#[macro_export]
macro_rules! trace_if {
    ($cond: expr, $($t: tt)*) => {
        if $cond {
            log::trace!($($t)*);
        }
    };
}

pub mod bytecode;
pub mod bytecompiler;
pub mod common;
pub mod frontend;
pub mod fullcodegen;
pub mod interpreter;
pub mod jit;
pub mod runtime;
