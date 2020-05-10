pub mod func;
pub mod lir;
pub mod osr;
pub mod types;
use crate::runtime::value::*;
pub enum JITResult {
    Ok(Value),
    Err(Value),
    OSRExit,
}
