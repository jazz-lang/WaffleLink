pub static mut SWEEPER_SAFEPOINT_PAGE: usize = 0;

#[inline(always)]
fn sweeper_safepoint() {
    unsafe {
        std::ptr::read_volatile(SWEEPER_SAFEPOINT_PAGE as *const usize);
    }
}

pub struct Sweeper {}

fn sweeper_worker(s: &Sweeper) {}
