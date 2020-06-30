macro_rules! offset_of {
    ($ty: ty, $field: ident) => {
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    };
}

pub mod builtins;
pub mod gc;
pub mod jit;
pub mod object;
pub mod pure_nan;
pub mod stack;
pub mod value;
pub mod vtable;
