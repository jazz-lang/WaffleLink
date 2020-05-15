#![allow(unused)]

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

#[macro_export]
macro_rules! unwrap {
    ($e: expr) => {
        match $e {
            Ok(x) => x,
            _ => unreachable!(),
        }
    };
}

pub mod assembler;
pub mod bytecode;
pub mod bytecompiler;
pub mod common;
pub mod frontend;
pub mod fullcodegen;
pub mod heap;
pub mod interpreter;
pub mod jit;
pub mod runtime;

#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;
pub use common::rc::Rc;
