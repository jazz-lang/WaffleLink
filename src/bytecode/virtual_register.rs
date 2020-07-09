use crate::interpreter::callframe::*;
use crate::interpreter::register::Register;
/// Register numbers used in bytecode operations have different meaning according to their ranges:
///      0x80000000-0xFFFFFFFF  Negative indices from the CallFrame pointer are entries in the call frame.
///      0x00000000-0x3FFFFFFF  Forwards indices from the CallFrame pointer are local vars and temporaries with the function's callframe.
///      0x40000000-0x7FFFFFFF  Positive indices from 0x40000000 specify entries in the constant pool on the CodeBlock.
pub const FIRST_CONSTANT_REGISTER_INDEX: i32 = 0x4000000;
pub const FIRST_CONSTANT_REGISTER_INDEX8: i32 = 16;
pub const FIRST_CONSTANT_REGISTER_INDEX16: i32 = 64;
pub const FIRST_CONSTANT_REGISTER_INDEX32: i32 = FIRST_CONSTANT_REGISTER_INDEX;

pub const fn is_local(operand: i32) -> bool {
    operand < 0
}

pub const fn is_argument(operand: i32) -> bool {
    operand >= 0
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct VirtualRegister {
    virtual_register: i32,
}

impl VirtualRegister {
    pub const INVALID: VirtualRegister = VirtualRegister {
        virtual_register: 0x3fffffff,
    };
    const fn local_to_operand(local: i32) -> i32 {
        -1 - local
    }

    const fn operand_to_local(operand: i32) -> i32 {
        -1 - operand
    }

    const fn operand_to_argument(operand: i32) -> i32 {
        operand - CallFrame::this_argument_offset()
    }

    const fn argument_to_operand(arg: i32) -> i32 {
        arg + CallFrame::this_argument_offset()
    }

    pub const fn offset(self) -> i32 {
        self.virtual_register
    }

    pub const fn offset_in_bytes(self) -> i32 {
        self.virtual_register * std::mem::size_of::<Register>() as i32
    }

    pub const fn to_local(self) -> i32 {
        Self::operand_to_local(self.virtual_register)
    }

    pub const fn to_argument(self) -> i32 {
        Self::operand_to_argument(self.virtual_register)
    }
    pub const fn new_constant_index(i: i32) -> Self {
        Self {
            virtual_register: i + FIRST_CONSTANT_REGISTER_INDEX,
        }
    }
    pub const fn to_constant_index(self) -> i32 {
        self.virtual_register - FIRST_CONSTANT_REGISTER_INDEX
    }

    pub const fn is_valid(self) -> bool {
        self.virtual_register != Self::INVALID.virtual_register
    }

    pub const fn is_local(self) -> bool {
        is_local(self.virtual_register)
    }

    pub const fn is_argument(self) -> bool {
        is_argument(self.virtual_register)
    }

    pub const fn is_header(self) -> bool {
        (self.virtual_register >= 0) & (self.virtual_register < CallFrameSlot::ThisArgument as i32)
    }

    pub const fn is_constant(self) -> bool {
        self.virtual_register >= FIRST_CONSTANT_REGISTER_INDEX
    }
}

#[inline(always)]
pub const fn virtual_register_for_local(local: i32) -> VirtualRegister {
    return VirtualRegister {
        virtual_register: VirtualRegister::local_to_operand(local),
    };
}

#[inline(always)]
pub const fn virtual_register_for_argument_including_this(
    argument: i32,
    offset: i32,
) -> VirtualRegister {
    VirtualRegister {
        virtual_register: VirtualRegister::argument_to_operand(argument) + offset,
    }
}

use std::fmt;

impl fmt::Display for VirtualRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.is_valid() {
            return write!(f, "<invalid>");
        }
        if self.is_header() {
            if self.virtual_register == CallFrameSlot::CodeBlock as i32 {
                write!(f, "codeBlock")?;
                return Ok(());
            } else if self.virtual_register == CallFrameSlot::Callee as i32 {
                write!(f, "callee")?;
                return Ok(());
            } else {
                #[cfg(target_pointer_width = "64")]
                {
                    if self.virtual_register == 0 {
                        write!(f, "callerFrame")?;
                        return Ok(());
                    } else if self.virtual_register == 1 {
                        write!(f, "returnPC")?;
                        return Ok(());
                    }
                }
                #[cfg(target_pointer_width = "32")]
                {
                    if self.virtual_register == 0 {
                        write!(f, "callerFrameAndPc")?;
                        return Ok(());
                    }
                }
            }
        }
        if self.is_constant() {
            return write!(f, "const{}", self.to_constant_index());
        }

        if self.is_argument() {
            if self.to_argument() == 0 {
                return write!(f, "this");
            } else {
                return write!(f, "arg{}", self.to_argument());
            }
        }

        write!(f, "loc{}", self.to_local())
    }
}

impl fmt::Debug for VirtualRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
