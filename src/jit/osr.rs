use std::collections::HashMap;
pub struct OSR {
    /// Map from basic block id to label in assembly
    pub labels: HashMap<u32, OSREntry>,
}

/// OSR Entry used for entering specific blocks of code.
///
/// OSR might be used to enter some specific bytecode impl in interpreter
/// or some specific code in assembly.
///
#[repr(C)]
pub struct OSREntry {
    pub to_ip: u32,
    pub to_bp: u32,

}

impl OSREntry {
    pub fn jit_jmp_addr(&self) -> u32 {
        self.to_ip
    }

    pub fn ip(&self) -> u32 {
        self.to_ip
    }

    pub fn bp(&self) -> u32 {
        self.to_bp
    }
}


#[derive(Clone)]
pub struct OSRTable {
    pub labels: Vec<usize>,
}
