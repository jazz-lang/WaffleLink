use crate::threading::*;
use minwinbase::*;
use winapi::um::errhandlingapi::*;
use winapi::um::winnt::*;
use winapi::um::*;
use winapi::vc::excpt::*;
pub unsafe extern "cdecl" fn crt_sig_handler(sig: i32, _num: i32) {
    libc::raise(sig);
}
pub unsafe extern "system" fn exception_handler(
    exception_info: *mut EXCEPTION_POINTERS,
) -> libc::c_long {
    let sp = false;
    let ei = &mut *exception_info;
    let er = &*ei.ExceptionRecord;
    if er.ExceptionFlags == 0 {
        match er.ExceptionCode {
            EXCEPTION_ACCESS_VIOLATION => {
                if er.ExceptionInformation[1] == crate::safepoint::SAFEPOINT_PAGE as _ {
                    let tls = get_tls_state();
                    tls.stack_end = &sp as *const bool as *mut u8;
                    set_gc_and_wait();

                    return EXCEPTION_CONTINUE_EXECUTION;
                }
            }
            _ => return EXCEPTION_CONTINUE_SEARCH,
        }
    }
    EXCEPTION_CONTINUE_SEARCH
}

pub fn install_default_signal_handlers() {
    unsafe {
        if libc::signal(libc::SIGSEGV, crt_sig_handler as _) == libc::SIG_ERR as usize {
            panic!("can't set signal handler");
        }
        SetUnhandledExceptionFilter(Some(exception_handler));
    }
}
