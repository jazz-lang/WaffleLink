pub mod lir;
pub mod osr;
use crate::runtime::value::*;
pub enum JITResult {
    Ok(Value),
    Err(Value),
    OSRExit,
}
