pub struct CallerFrameAndPc {
    caller_frame: *mut u8,
    return_pc: *mut u8,
}

impl CallerFrameAndPc {
    pub const SIZE_IN_REGISTERS: usize =
        2 * std::mem::size_of::<usize>() / std::mem::size_of::<usize>();
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(i32)]
pub enum CallFrameSlot {
    CodeBlock = CallerFrameAndPc::SIZE_IN_REGISTERS as i32,
    Callee = Self::CodeBlock as i32 + 1,
    ArgumentCountIncludingThis = Self::Callee as i32 + 1,
    ThisArgument = Self::ArgumentCountIncludingThis as i32 + 1,
    FirstArgument = Self::ThisArgument as i32 + 1,
}
use crate::bytecode::*;
use crate::interpreter::register::*;
use crate::value::Value;
pub struct CallFrame(Register);

impl CallFrame {
    pub fn caller_frame_and_pc(&self) -> &mut CallerFrameAndPc {
        unsafe { &mut *std::mem::transmute::<_, *mut CallerFrameAndPc>(self) }
    }

    pub fn arg_idx_for_register(&self, reg: *mut u8) -> usize {
        let offset = reg as usize - self.registers() as usize;
        let idx = offset - CallFrameSlot::FirstArgument as usize;
        idx
    }

    pub fn offset_for(x: usize) -> i32 {
        CallFrameSlot::FirstArgument as i32 + x as i32 - 1
    }

    pub fn argument_count_including_this(&self) -> usize {
        unsafe {
            (*self
                .registers()
                .offset(CallFrameSlot::ArgumentCountIncludingThis as _))
            .payload() as _
        }
    }

    pub fn argument_count(&self) -> usize {
        self.argument_count_including_this() - 1
    }

    pub const fn argument_offset(arg: i32) -> i32 {
        CallFrameSlot::FirstArgument as i32 + arg
    }

    pub const fn argument_offset_including_this(arg: i32) -> i32 {
        CallFrameSlot::ThisArgument as i32 + arg
    }

    pub fn address_of_arguments_start(&self) -> *mut Value {
        unsafe {
            self.registers()
                .offset(Self::argument_offset(0) as _)
                .cast()
        }
    }

    pub fn argument(&self, arg: usize) -> Value {
        if arg >= self.argument_count() {
            return Value::undefined();
        }
        unsafe { self.get_argument_unsafe(arg) }
    }

    pub unsafe fn get_argument_unsafe(&self, idx: usize) -> Value {
        return (*self.registers().offset(idx as _)).u.value;
    }
    pub fn registers(&self) -> *mut Register {
        self as *const Self as *mut _
    }
    pub const fn this_argument_offset() -> i32 {
        Self::argument_offset_including_this(0)
    }

    pub fn this(&self) -> Value {
        unsafe {
            (*self.registers().offset(Self::this_argument_offset() as _))
                .u
                .value
        }
    }

    pub fn set_this(&self, val: Value) {
        unsafe {
            self.registers()
                .offset(Self::this_argument_offset() as _)
                .cast::<Value>()
                .write(val);
        }
    }

    pub fn set_argument_count_including_this(&self, count: i32) {
        unsafe {
            *(&mut *self
                .registers()
                .offset(CallFrameSlot::ArgumentCountIncludingThis as _))
                .payload_mut() = count;
        }
    }

    pub fn set_return_pc(&self, addr: *mut u8) {
        self.caller_frame_and_pc().return_pc = addr;
    }

    pub fn return_pc(&self) -> *mut u8 {
        self.caller_frame_and_pc().return_pc
    }

    pub fn return_pc_offset_of() -> usize {
        offset_of!(CallerFrameAndPc, return_pc)
    }

    pub fn caller_frame(&self) -> *mut CallFrame {
        self.caller_frame_and_pc().caller_frame as *mut _
    }

    pub fn code_block(&self) -> *mut CodeBlock {
        return unsafe {
            self.registers()
                .offset(CallFrameSlot::CodeBlock as _)
                .read()
                .u
                .code_block
        };
    }

    pub fn code_block_ref(&self) -> Option<&mut CodeBlock> {
        if self.code_block().is_null() {
            return None;
        } else {
            return Some(unsafe { &mut *self.code_block() });
        }
    }

    pub fn top_of_frame(&self) -> *mut Register {
        if self.code_block().is_null() {
            return self.registers();
        }
        unsafe {
            self.registers()
                .offset((&*self.code_block()).stack_pointer_offset() as _)
        }
    }
}
