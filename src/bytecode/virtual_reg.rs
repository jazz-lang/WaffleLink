#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct VirtualRegister(i32);

impl VirtualRegister {
    pub const INVALID_VIRTUAL_REGISTER: i32 = 0x3fffffff;
    pub fn is_local(self) -> bool {
        self.0 < 0
    }

    pub fn is_argument(self) -> bool {
        self.0 >= 0
    }
}

use std::fmt;

impl fmt::Display for VirtualRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_local() {
            write!(f, "loc{}", -self.0)
        } else {
            write!(f, "arg{}", self.0)
        }
    }
}
