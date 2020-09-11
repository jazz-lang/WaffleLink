pub mod atomics;
pub mod fast_bitvec;
pub mod uniset;
pub trait Bool {
    const RES: bool;
}

pub struct True;
pub struct False;

impl Bool for True {
    const RES: bool = true;
}
impl Bool for False {
    const RES: bool = false;
}
