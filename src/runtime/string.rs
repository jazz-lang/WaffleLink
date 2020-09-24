use crate::prelude::*;
#[repr(C)]
pub struct WString {
    pub(crate) cell_type: CellType,
    pub string: String,
}

impl WString {
    pub fn new(isolate: &RCIsolate, x: &str) -> Local<Self> {
        isolate.new_local(Self {
            cell_type: CellType::String,
            string: x.to_string(),
        })
    }
}

impl GcObject for WString {}
