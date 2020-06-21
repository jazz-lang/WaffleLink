use crate::object::*;
use crate::value::Value;
use crate::*;
use gc::RootedCell;

#[repr(C)]
pub struct Module {
    pub header: WaffleTypeHeader,
    pub name: Value,
    pub constants: WaffleCellPointer<WaffleArray>,
    pub functions: WaffleCellPointer<WaffleArray>,
    pub entry: Value,
}

impl WaffleCellTrait for Module {
    const TYPE: WaffleType = WaffleType::Module;
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

impl Module {
    pub fn new_empty(fcount: usize, constants: &[Value]) -> RootedCell<Self> {
        let module: RootedCell<Module> = VM
            .state
            .heap
            .allocate(WaffleType::Module, std::mem::size_of::<Self>())
            .unwrap();
        if constants.is_empty() {
            module.value_mut().constants = WaffleCellPointer::null();
        } else {
            let mut constants_r = WaffleArray::new(Value::undefined(), constants.len());
            for ix in 0..constants.len() {
                *constants_r.value_mut().at_ref_mut(ix) = constants[ix];
            }
            module.value_mut().constants = constants_r.to_heap();
        }
        module.value_mut().entry = Value::undefined();
        module.value_mut().name = Value::undefined();
        module.value_mut().functions = WaffleArray::new(Value::undefined(), fcount).to_heap();
        module
    }
}
