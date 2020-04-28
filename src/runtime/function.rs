use super::cell::*;
use super::frame::*;
use super::value::*;
use crate::arc::ArcWithoutWeak as Arc;
use crate::bytecode::op::*;
use crate::common::ptr::*;
use smallvec::SmallVec;
pub struct Function {
    pub name: Value,
    /// Feedback vector, used to store type information when interpreting
    /// and later JIT uses type information from feedback vector.
    ///
    /// Feedback vector is allocated or copied per process.
    pub feedback_vector: Vec<FeedBack>,
    pub can_jit: bool,
    /// Hotness threshold of this function.
    ///
    /// Hotness is incremented by 50 on each function invocation and by 1 on each loop iteration.
    /// Threeshold needed until JIT:
    ///
    /// 500: Simple JIT
    ///
    /// 1000: Easy JIT
    ///
    /// 100000: Full JIT
    pub threshold: usize,
    pub constants: Vec<Value>,
    pub upvalues: Vec<Value>,
    pub code: FunctionCode,
    pub module: Value,
    pub simple_jit: Option<extern "C" fn(&mut Frame) -> Result<Value, Value>>,
    pub full_jit: Option<extern "C" fn(&mut Frame) -> Result<Value, Value>>,
}

impl Function {
    #[inline(always)]
    pub fn get_bytecode_unchecked(&self) -> &Vec<BasicBlock> {
        unsafe {
            match &self.code {
                FunctionCode::Bytecode(bc) => bc,
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }
    #[inline(always)]
    pub fn get_bytecode_unchecked_mut(&mut self) -> &mut Vec<BasicBlock> {
        unsafe {
            match &mut self.code {
                FunctionCode::Bytecode(bc) => bc,
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }
}

pub enum FeedBack {
    None,
    Cache(Arc<super::map::Map>, u32, u16),
    Count(usize),
    TypeInfo(SmallVec<[Type; 3]>),
}

pub enum Type {
    Int32,
    Number,
    NaNNumber,
    NaNInfNumber,
    Boolean,
    String,
    Object,
    Array,
    Function,
    Generator,
    Null,
    Undefined,
}

pub enum FunctionCode {
    /// Native function.
    Native(extern "C" fn(&mut Frame) -> Result<Value, Value>),
    Bytecode(Vec<BasicBlock>),
}

pub struct BasicBlock {
    pub code: Vec<OpV>,
}
