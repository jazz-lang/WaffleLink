#![allow(missing_docs)]

use std::sync::atomic::{fence, Ordering};

pub fn compiler_fence() {
    std::sync::atomic::compiler_fence(Ordering::SeqCst);
}

pub fn load_load_fence() {
    fence(Ordering::SeqCst);
}

pub fn load_store_fence() {
    fence(Ordering::SeqCst);
}

pub fn store_load_fence() {
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        fence(Ordering::SeqCst);
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        fence(Ordering::AcqRel);
    }
}

pub fn store_store_fence() {
    fence(Ordering::SeqCst);
}
