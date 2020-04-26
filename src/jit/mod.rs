//! All JIT tiers implementations.
//!
//! Currently JIT is performed on entire function rather than compiling some
//! hot parts of function but this might be solved by implementing OSR.
//!
//! ## OSR
//! What OSR does is allows switching between interpreter context and JIT code.
//! OSR might help in JIT compiling loops in functions.

pub mod simple;
