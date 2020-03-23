pub mod arc;
pub mod cell;
pub mod threads;
pub mod value;
pub struct Runtime {
    pub threads: threads::Threads,
}

lazy_static::lazy_static! {
    pub static ref RT: Runtime = unimplemented!();
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            threads: threads::Threads::new(),
        }
    }
}
