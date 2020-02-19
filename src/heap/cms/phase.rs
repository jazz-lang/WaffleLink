#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i8)]
pub enum Phase {
    First = 0x00,
    Second,
    Third,
    Tracing,
    Fourth,
    Sweep,
}

impl Phase {
    #[inline]
    pub fn advance(&mut self) -> Phase {
        *self = unsafe { std::mem::transmute((*self as i8 + 1) % 6) };
        *self
    }
    #[inline]
    pub fn prev(&self) -> Self {
        unsafe { std::mem::transmute((*self as i8 - 1) % 6) }
    }

    #[inline]
    pub fn snooping(&self) -> bool {
        *self as i8 <= 1
    }

    #[inline]
    pub fn tracing(&self) -> bool {
        let val = *self as i8;
        val >= 1 && val <= 3
    }
}
