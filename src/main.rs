use wafflelink::safepoint::*;
use wafflelink::signals::*;
use wafflelink::threading::*;
fn main() {
    install_default_signal_handlers();
    unsafe {
        safepoint_init();
    }
    register_main_thread();

    let t = spawn_rt_thread(|| {
        let tls = get_tls_state();
        loop {
            tls.yieldpoint();
        }
    });

    std::thread::sleep(std::time::Duration::from_millis(50));

    safepoint_start_gc();

    let _lock = safepoint_wait_for_the_world();
    println!("suspended {} threads(s)", _lock.len());
    safepoint_end_gc();
}
