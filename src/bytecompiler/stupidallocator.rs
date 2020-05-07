use crate::bytecode;
use bv::BitVec;
use bytecode::virtual_reg::*;

pub type RegSet = BitVec<usize>;

pub struct StupidAllocator {
    ntemps: i32,
    nlocals: i32,
    state: RegSet,
    context: Vec<RegSet>,
}

const MAX_REGS: i32 = 16000;
