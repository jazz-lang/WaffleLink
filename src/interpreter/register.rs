use crate::bytecode::CodeBlock;
use crate::interpreter::*;
use crate::value::*;
use callframe::*;
pub union RegisterDescr {
    pub value: Value,
    pub callframe: *mut CallFrame,
    pub code_block: *mut CodeBlock,
    pub encoded_value: EncodedValueDescriptor,
    pub number: f64,
    pub integer: i64,
}

pub struct Register {
    pub u: RegisterDescr,
}

impl Register {
    pub fn payload(&self) -> i32 {
        unsafe { self.u.value.u.as_bits.payload }
    }

    pub fn tag(&self) -> i32 {
        unsafe { self.u.value.u.as_bits.tag }
    }
    pub fn payload_mut(&mut self) -> &mut i32 {
        unsafe { &mut self.u.value.u.as_bits.payload }
    }

    pub fn tag_mut(&mut self) -> &mut i32 {
        unsafe { &mut self.u.value.u.as_bits.tag }
    }

    pub fn pointer(&self) -> *mut u8 {
        #[cfg(feature = "value64")]
        {
            return unsafe { self.u.value.u.as_int64 as *mut u8 };
        }
        #[cfg(feature = "value32-64")]
        {
            return self.payload() as *mut u8;
        }
    }
}
