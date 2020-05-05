extern crate cgc_single_threaded as cgc;

#[macro_export]
macro_rules! offset_of {
    ($ty: ty, $field: ident) => {
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    };
}

pub mod bytecode;
pub mod interpreter;
pub mod runtime;
