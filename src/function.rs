use crate::*;
use gc::*;
use object::*;
use value::Value;

#[repr(C)]
pub struct FunctionResult {
    error: bool,
    value: Value,
}

pub type FunctionPtr = extern "C" fn(Value, Value) -> FunctionResult;

#[repr(C)]
pub struct Function {
    pub header: WaffleTypeHeader,
    pub name: Value,
    pub env: WaffleCellPointer<WaffleArray>,
    pub fptr: Option<FunctionPtr>,
}

impl WaffleCellTrait for Function {
    const TYPE: WaffleType = WaffleType::Function;
    fn ty(&self) -> Option<WaffleType> {
        Some(Self::TYPE)
    }

    fn header(&self) -> &WaffleTypeHeader {
        &self.header
    }

    fn header_mut(&mut self) -> &mut WaffleTypeHeader {
        &mut self.header
    }
}
