pub mod cell;
pub mod pure_nan;
pub mod transition_map;
pub mod value;
use crate::common::rc::Rc;
use std::cell::RefCell;
thread_local! {
    pub static RT: RefCell<Option<Rc<Runtime>>> = RefCell::new(Some(Rc::new(Runtime::new())));
}

pub fn get_rt() -> Rc<Runtime> {
    RT.with(|rt| rt.borrow().as_ref().unwrap().clone())
}

pub struct Runtime {
    pub heap: cgc::heap::Heap,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            heap: cgc::heap::Heap::new(16 * 1024, 32 * 1024, true),
        }
    }
}
