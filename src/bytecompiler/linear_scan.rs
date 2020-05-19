use crate::bytecode;
use bytecode::def::*;
use bytecode::virtual_reg::*;
pub fn is_precolored(x: VirtualRegister) -> bool {
    if x.is_local() {
        x.to_local() < 256
    } else {
        false
    }
}

pub type Interval = std::ops::Range<usize>;

use hashlink::LinkedHashSet;

pub struct TmpData {
    pub interval: Interval,
    pub possible_regs: LinkedHashSet<VirtualRegister>,
    pub assigned: VirtualRegister,
    pub did_build_possible_regs: bool,
}

pub struct Clobber {
    pub index: usize,
    pub regs: LinkedHashSet<VirtualRegister>,
}

const FIRST_PHASE: usize = 0;
const SECOND_PHASE: usize = 1;
use indexmap::*;

use crate::common::bitmap::*;

pub struct RegisterSet {
    bits: BitMap,
}

impl RegisterSet {
    const SPEC: usize = 256;

    pub fn for_each(&self, mut f: impl FnMut(VirtualRegister)) {
        self.bits
            .for_each_set_bit(|x| f(VirtualRegister::tmp(x as _)))
    }

    pub fn is_deleted_value(&self) -> bool {
        self.bits.get(256) && self.bits.get(0)
    }

    pub fn is_empty_value(&self) -> bool {
        self.bits.get(256) && !self.bits.get(0)
    }

    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }
    pub fn subsumes(&self, other: &Self) -> bool {
        self.bits.subsumes(&other.bits)
    }

    pub fn exlude(&mut self, other: &Self) {
        self.bits.exlude(&other.bits)
    }

    pub fn merge(&mut self, other: &Self) {
        self.bits.merge(&other.bits)
    }
    pub fn get(&self, x: VirtualRegister) -> bool {
        self.bits.get(x.to_local() as _)
    }

    pub fn add(&mut self, x: VirtualRegister) -> bool {
        !self.bits.test_and_set(x.to_local() as _)
    }

    pub fn remove(&mut self, x: VirtualRegister) -> bool {
        !self.bits.test_and_clear(x.to_local() as _)
    }

    pub fn set_val(&mut self, x: VirtualRegister, val: bool) {
        self.bits.set_val(x.to_local() as _, val);
    }
    pub fn set(&mut self, x: VirtualRegister) {
        self.set_val(x, true);
    }
    pub fn set_all<'a>(&mut self, x: impl std::iter::Iterator<Item = &'a VirtualRegister>) {
        for x in x {
            self.set_val(*x, true);
        }
    }
}

pub struct LinearScan<'a> {
    code: &'a mut bytecode::CodeBlock,
    registers: Vec<VirtualRegister>,
    register_set: RegisterSet,
    unified: RegisterSet,
    start_index: IndexMap<u32, usize>,
    map: IndexMap<VirtualRegister, TmpData>,
    clobbers: Vec<Clobber>,
    tmps: Vec<VirtualRegister>,
    active: std::collections::VecDeque<VirtualRegister>,
    active_regs: RegisterSet,
}

impl<'a> LinearScan<'a> {
    fn build_register_set(&mut self) {
        let mut bank = vec![];
        for i in 0..255 {
            let reg = VirtualRegister::tmp(i);
            bank.push(reg);
        }
        self.registers = bank.clone();
        self.register_set.set_all(bank.iter());
        //self.unified.merge(&bank);
    }
}
