#![feature(asm)]
#![feature(core_intrinsics)]
#[macro_export]
macro_rules! offset_of {
    ($ty: ty, $field: ident) => {
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    };
}
#[macro_export]
macro_rules! __hidden_handling_cases {
    // hide this case from the API
    (else $e: expr) => {$e};

    ($cond : expr => $if_true: expr;
        $(elif $cond2: expr => $if_t2:expr;)*
        else $if_false:expr
    ) => {
        [__hidden_handling_cases!($($cond2 => $if_t2; )elif * else $if_false), $if_true][$cond as bool as usize]
    };
}

#[macro_export]
macro_rules! const_if {
    ($cond: expr => $if_true:expr;$if_false: expr) => {
        [$if_false, $if_true][(!!$cond)  as usize]
    };

    // delegate to private macro
    ($cond : expr => $if_true: expr;
        $(elif $cond2: expr => $if_t2:expr;)*
        else $if_false:expr
    ) => {
        __hidden_handling_cases!($cond => $if_true; $(elif $cond2 => $if_t2;)* else $if_false)
    };
}
#[macro_use]
macro_rules! offset_of_field_fn {
    ($name: ident) => {
        paste::item!(
            pub fn [<offset_of_ $name>] () -> usize {
                offset_of!(Self,$name)
            }
        );
    };
}
#[macro_export]
macro_rules! likely {
    ($e: expr) => {
        std::intrinsics::likely($e)
    };
}
macro_rules! unlikely {
    ($e: expr) => {
        std::intrinsics::unlikely($e)
    };
}
pub use std::intrinsics::{likely, unlikely};
pub mod heap;
pub mod runtime;
