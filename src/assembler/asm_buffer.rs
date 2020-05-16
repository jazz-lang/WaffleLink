//! Assembly buffer provides buffer for emitting assembly and patchpoint like functionality.

use crate::common::Address;

pub enum AsmBuffer {
    Vec(Vec<u8>),
    Patch {
        current: Address,
        start: Address,
        end: Address,
    }
}

impl AsmBuffer {
    pub fn push(&mut self,x: u8) {
        match self {
            AsmBuffer::Vec(v) => v.push(x),
            AsmBuffer::Patch {
                current,start,
                end
            } => {
                let addr = current.offset(1).to_mut_ptr::<u8>();
                unsafe {
                    addr.write(x);
                }
                *current = Address::from_ptr(addr);
                if current > end {
                    panic!("cannot patch more!");
                }
            }
        }
    }

    pub fn as_ref(&self) -> &[u8] {
        match self {
            AsmBuffer::Vec(v) => v,
            AsmBuffer::Patch {
                start,end,..
            } => {
                let size = end.to_usize() - start.to_usize();
                unsafe {
                    std::slice::from_raw_parts(start.to_ptr::<u8>(),size)
                }
            }
        }
    }
    pub fn as_mut(&mut self) -> &mut [u8] {
        match self {
            AsmBuffer::Vec(v) => v,
            AsmBuffer::Patch {
                start,end,..
            } => {
                let size = end.to_usize() - start.to_usize();
                unsafe {
                    std::slice::from_raw_parts_mut(start.to_mut_ptr::<u8>(),size)
                }
            }
        }
    }

    pub fn as_vec(self)-> Vec<u8> {
        match self {
            AsmBuffer::Vec(v) => v,
            _ => unreachable!()
        }
    }
    pub fn as_vec_mut(&mut self)-> &mut Vec<u8> {
        match self {
            AsmBuffer::Vec(v) => v,
            _ => unreachable!()
        }
    }

    pub fn len(&self) -> usize {
        match self {
            AsmBuffer::Vec(v) => v.len(),
            AsmBuffer::Patch {
                start,current,..
            } => {
                current.to_usize() - start.to_usize()
            }
        }
    }
}