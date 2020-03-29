//! Cake Garbage Collector.
//!
//! Cake is a concurrent mark&sweep garbage collector with optional evacutaion.
//! GC is highly inspired by WebKit's Riptide garbage collector and borrows a
//! lot ideas from it.
//!
//! Cake has concurrent mark&sweep and also optional evacuation for blocks. When
//! evacuation is triggered then full STW is used because we cannot properly move objects
//! in concurrent mode (there are some read barriers,but we do not implement them).

pub mod cell_state;
pub mod freelist;
pub mod heap_cell;

pub mod prelude {
    pub use super::cell_state::*;
    pub use super::freelist::*;
    pub use super::heap_cell::*;
}
