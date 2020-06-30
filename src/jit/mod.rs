#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod jit_x86;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use jit_x86::*;
