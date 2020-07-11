use super::*;

pub struct RegisterID {
    rc: u32,
    virtual_register: VirtualRegister,
    is_tmp: bool,
}

impl RegisterID {
    pub fn ref_(&mut self) {
        self.rc += 1;
    }

    pub fn deref(&mut self) {
        self.rc -= 1;
    }
    pub fn rc(&self) -> u32 {
        self.rc
    }

    pub fn is_tmp(&self) -> bool {
        self.is_tmp
    }

    pub fn set_index(&mut self, reg: VirtualRegister) {
        self.virtual_register = reg;
    }
    pub fn new_idx(i: i32) -> Self {
        Self {
            is_tmp: false,
            virtual_register: VirtualRegister {
                virtual_register: i,
            },
            rc: 0,
        }
    }
    pub fn new(x: VirtualRegister) -> Self {
        Self {
            is_tmp: false,
            rc: 0,
            virtual_register: x,
        }
    }

    pub fn set_tmp(&mut self) {
        self.is_tmp = true;
    }
}
