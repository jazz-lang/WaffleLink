pub mod arc;
pub mod thread;
pub mod value;

pub struct Runtime {
    pub threads: thread::Threads,
}

lazy_static::lazy_static! {
    pub static ref RT: Runtime = unimplemented!();
}
