#[cfg(target_family = "unix")]
pub mod unix;
#[cfg(target_family = "unix")]
pub use unix::*;

#[cfg(target_family = "windows")]
pub mod windows;
#[cfg(target_family = "windows")]
pub use windows::*;

// TODO: Allow for GC to offload some of work to suspended threads...
