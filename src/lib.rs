use std::sync::Arc;

pub mod gc;
pub mod module;
pub mod object;
pub mod pure_nan;
pub mod value;
pub mod vtable;
use gc::GlobalAllocator;
pub struct VirtualMachine {
    pub state: Arc<GlobalState>,
}

pub struct GlobalState {}
#[cfg(target_pointer_width = "32")]
compile_error!("Cannot build on OS/Architecture with 32 bit pointers");
