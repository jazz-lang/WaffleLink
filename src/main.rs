use capstone::prelude::*;
use jit::{jit64::add_generator::*, jit64::*, *};
use linkbuffer::*;
use masm::*;
use wafflelink::bytecode::*;
use wafflelink::jit;
use x86_assembler::*;
use x86masm::*;
fn main() {
    let mut jit = JIT::new(&[
        Ins::TryCatch(1, 2),
        Ins::Add(0, 1, 2),
        Ins::GetException(0),
        Ins::Safepoint,
    ]);

    //let mut add = AddGenerator::new(T0, T1, T2, T3, FT0, FT1);
    //add.generate_fastpath(&mut jit);
    jit.compile_bytecode();
    jit.masm.ret();
    let mut m = Memory::new();
    jit.finalize(&mut m, true);
}
