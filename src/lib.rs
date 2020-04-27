#![feature(const_generics)]
#![allow(incomplete_features)]
#![allow(non_camel_case_types)]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(const_raw_ptr_deref)]
#![allow(const_err)]
#![feature(naked_functions)]
#![feature(untagged_unions)]
#![feature(global_asm)]
#![allow(improper_ctypes)]
#![feature(arbitrary_self_types)]
#![feature(llvm_asm)]
#[macro_export]
macro_rules! offset_of {
    ($ty: ty, $field: ident) => {
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    };
}
pub mod arc;
pub mod assembler;
pub mod bytecode;
pub mod common;
pub mod heap;
pub mod interpreter;

#[cfg(feature = "jit")]
pub mod jit;
pub mod runtime;
