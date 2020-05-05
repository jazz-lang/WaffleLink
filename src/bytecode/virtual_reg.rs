#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct VirtualRegister(pub i32);
use super::*;
impl VirtualRegister {
    pub const INVALID_VIRTUAL_REGISTER: i32 = 0x3fffffff;
    pub fn is_local(self) -> bool {
        self.0 < 0
    }

    pub fn is_argument(self) -> bool {
        self.0 >= 0
    }

    pub fn is_constant(self) -> bool {
        self.0 >= FIRST_CONSTNAT_REG_INDEX
    }

    pub fn to_local(self) -> i32 {
        -1 - self.0
    }

    pub fn to_argument(self) -> i32 {
        self.0 - 0x80000000u32 as i32
    }

    pub fn to_constant(self) -> i32 {
        self.0 - FIRST_CONSTNAT_REG_INDEX
    }

    pub fn tmp(x: i32) -> Self {
        Self(-1 - x)
    }

    pub fn argument(x: i32) -> Self {
        Self(x + 0x80000000u32 as i32)
    }

    pub fn constant(x: i32) -> Self {
        Self(x + FIRST_CONSTNAT_REG_INDEX)
    }
}

use std::fmt;

impl fmt::Display for VirtualRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_local() {
            write!(f, "loc{}", self.to_local())
        } else if self.is_argument() {
            write!(f, "arg{}", self.to_argument())
        } else {
            write!(f, "id{}", self.to_constant())
        }
    }
}
