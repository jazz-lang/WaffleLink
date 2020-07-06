#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod jit_x86;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use jit_x86::*;
#[cfg(target_pointer_width = "64")]
pub mod jit64;
#[cfg(target_pointer_width = "64")]
pub mod tail_call64;

use crate::builtins::WResult;
use crate::stack::callframe::CallFrame;
pub type JITFunction = extern "C" fn(&mut CallFrame) -> WResult;
pub type JITTrampoline = extern "C" fn(&mut CallFrame, usize) -> WResult;

pub extern "C" fn safepoint_slow_path(_sp: *mut u8) {}
