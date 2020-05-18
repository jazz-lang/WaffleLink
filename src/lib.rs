#![allow(unused)]
#![feature(llvm_asm)]
#![feature(test)]
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
#[cfg(target_arch = "x86_64")]
macro_rules! call {
    (before ) => {};
    (after) => {};
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

#[cfg(test)]
mod tests {

    extern crate test;
    use test::Bencher;

    use crate::bytecompiler::*;
    use crate::frontend::*;
    use crate::fullcodegen::FullCodegen;
    use crate::heap::api::*;
    use crate::interpreter::callstack::CallFrame;
    use crate::jit::JITResult;
    use crate::runtime::*;
    use parser::*;
    use reader::*;
    use value::*;

    #[bench]
    fn jit_loop(b: &mut Bencher) {
        let mut rt = Runtime::new(Configs::default());
        let reader = Reader::from_string(
            "
var i = 0 
while i < 100000 {
    i = i + 1
}
return i
",
        );
        let mut ast = vec![];
        let mut p = Parser::new(reader, &mut ast);
        if let Err(e) = p.parse() {
            eprintln!("{}", e);
            return;
        }
        let code = match compile(&mut rt, &ast) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        let f = function_from_codeblock(&mut rt, code.clone(), "<main>");
        b.iter(|| {
            let _ = rt.call(f, Value::undefined(), &[]);
        });
    }
    #[bench]
    fn interp_loop(b: &mut Bencher) {
        let mut rt = Runtime::new(Configs::default().no_jit());
        let reader = Reader::from_string(
            "
var i = 0 
while i < 100000 {
    i = i + 1
}
return i
",
        );
        let mut ast = vec![];
        let mut p = Parser::new(reader, &mut ast);
        if let Err(e) = p.parse() {
            eprintln!("{}", e);
            return;
        }
        let code = match compile(&mut rt, &ast) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        let f = function_from_codeblock(&mut rt, code.clone(), "<main>");
        b.iter(|| {
            let _ = rt.call(f, Value::undefined(), &[]);
        });
    }
}
