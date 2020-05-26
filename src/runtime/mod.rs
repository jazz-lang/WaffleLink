pub mod cell;
pub mod pure_nan;
pub mod scope;
pub mod transition_map;
pub mod value;
pub mod vtable;
use crate::common::rc::Rc;
use std::cell::RefCell;
thread_local! {
    pub static RT: RefCell<Option<Rc<Runtime>>> = RefCell::new(Some(Rc::new(Runtime::new())));
}

pub fn get_rt() -> Rc<Runtime> {
    RT.with(|rt| rt.borrow().as_ref().unwrap().clone())
}

pub struct Runtime {
    pub heap: crate::gc::Heap,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            heap: unimplemented!(),
        }
    }
}
