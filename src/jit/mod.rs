pub mod func;
pub mod lir;
pub mod osr;
pub mod types;
use crate::runtime::value::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub enum JITResult {
    Ok(Value),
    Err(Value),
    OSRExit,
}
