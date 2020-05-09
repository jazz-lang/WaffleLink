#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct VirtualRegister(pub i32);
use super::*;
impl VirtualRegister {
    pub const FIRST_AVAILABLE_LOCAL_REG: Self = Self::tmp(512);
    pub const INVALID_VIRTUAL_REGISTER: i32 = 0x3fffffff;
    pub const fn is_local(self) -> bool {
        self.0 < 0
    }

    pub const fn is_argument(self) -> bool {
        self.0 >= 0
    }

    pub const fn is_constant(self) -> bool {
        self.0 >= FIRST_CONSTNAT_REG_INDEX
    }

    pub const fn to_local(self) -> i32 {
        -1 - self.0
    }

    pub fn to_argument(self) -> i32 {
        self.0
    }

    pub const fn to_constant(self) -> i32 {
        self.0 - FIRST_CONSTNAT_REG_INDEX
    }

    pub const fn tmp(x: i32) -> Self {
        Self(-1 - x)
    }

    pub const fn argument(x: i32) -> Self {
        Self(x)
    }

    pub const fn constant(x: i32) -> Self {
        Self(x + FIRST_CONSTNAT_REG_INDEX)
    }

    pub fn is_real(self) -> bool {
        self.is_local() && self.to_local() < Self::FIRST_AVAILABLE_LOCAL_REG.to_local()
    }
}

use std::fmt;

impl fmt::Display for VirtualRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_local() {
            write!(f, "loc{}", self.to_local())
        } else if self.is_constant() {
            write!(f, "id{}", self.to_constant())
        } else {
            write!(f, "arg{}", self.to_argument())
        }
    }
}

use std::ops::*;

impl Add for VirtualRegister {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        if self.is_local() && rhs.is_local() {
            return VirtualRegister::tmp(self.to_local() + rhs.to_local());
        } else if self.is_argument() && rhs.is_argument() {
            return VirtualRegister::argument(self.to_argument() + rhs.to_local());
        }
        panic!();
    }
}
