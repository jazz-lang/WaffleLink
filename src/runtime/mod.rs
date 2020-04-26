pub mod cell;
pub mod constants;
pub mod frame;
pub mod function;
pub mod map;
pub mod module;
pub mod process;
pub mod symbol;
pub mod value;
pub struct RuntimeOpts {
    pub lazy_sweep: bool,
}

lazy_static::lazy_static! {
    pub static ref OPTIONS: RuntimeOpts = RuntimeOpts {
        lazy_sweep: true
    };
}
