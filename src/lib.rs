use std::sync::Arc;

pub mod gc;
pub mod object;
pub mod pure_nan;
pub mod value;
pub mod vtable;
use gc::GlobalAllocator;
pub struct VirtualMachine {
    pub state: Arc<GlobalState>,
}

pub struct GlobalState {}
