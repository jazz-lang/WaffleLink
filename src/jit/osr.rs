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
pub struct OSREntry {
    pub to_bp: usize,
    pub to_ip: usize,
}

impl OSREntry {
    pub fn jit_jmp_addr(&self) -> usize {
        self.to_ip
    }

    pub fn ip(&self) -> usize {
        self.to_ip
    }

    pub fn bp(&self) -> usize {
        self.to_bp
    }
}
