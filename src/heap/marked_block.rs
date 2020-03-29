use super::prelude::*;

pub struct MarkedBlock;

impl MarkedBlock {
    pub const ATOM_SIZE: usize = 16;
    pub const BLOCK_SIZE: usize = 16 * 1024;
    pub const BLOCK_MASK: usize = !(Self::BLOCK_SIZE - 1);
    pub const ATOMS_PER_BLOCK: usize = Self::BLOCK_SIZE / Self::ATOM_SIZE;
}

#[derive(Copy,Clone,PartialEq,Eq,Debug)]
#[repr(u8)]
pub enum SweepMode {
    SweepOnly,
    SweepToFreeList
}
