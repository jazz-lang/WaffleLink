use super::block::*;

/// Block size must be at least as large as the system page size.
pub const BLOCK_SIZE: usize = 32 * 1024;
/// Single atom size
pub const ATOM_SIZE: usize = 16;
/// Numbers of atoms per block
pub const ATOMS_PER_BLOCK: usize = BLOCK_SIZE / ATOM_SIZE;
/// Lower tiers maximum
pub const MAX_NUMBER_OF_LOWER_TIER_CELLS: usize = 8;
/// End atom offset
pub const END_ATOM: usize = (BLOCK_SIZE - core::mem::size_of::<BlockHeader>()) / ATOM_SIZE;
/// Block payload size
pub const PAYLOAD_SIZE: usize = END_ATOM * ATOM_SIZE;
/// Block header size
pub const FOOTER_SIZE: usize = BLOCK_SIZE - PAYLOAD_SIZE;
/// Atom alignment mask
pub const ATOM_ALIGNMENT_MASK: usize = ATOM_SIZE - 1;

pub const BITMAP_SIZE: usize = ATOMS_PER_BLOCK;
pub const BITS_IN_WORD: usize = core::mem::size_of::<usize>() * 8;
pub const NUMBER_OF_WORDS: usize = (BITMAP_SIZE + BITS_IN_WORD - 1) / BITS_IN_WORD;
