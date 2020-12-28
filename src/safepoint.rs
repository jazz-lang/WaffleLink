use crate::heap::*;
use crate::mutex::Mutex;
use crate::threading::*;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_family = "windows")]
use winapi::um::{memoryapi::*, winnt::*};
pub static mut SAFEPOINT_PAGE: *mut u8 = 0 as *mut _;
pub static mut SAFEPOINT_ENABLE_CNT: i8 = 0;
pub static SAFEPOINT_LOCK: Mutex = Mutex::new();
pub static GC_RUNNING: AtomicBool = AtomicBool::new(false);

unsafe fn enable_safepoint() {
    /*SAFEPOINT_ENABLE_CNT += 1;
    if SAFEPOINT_ENABLE_CNT - 1 != 0 {
        assert!(SAFEPOINT_ENABLE_CNT <= 2);
        return;
    }*/
    let pageaddr = SAFEPOINT_PAGE;
    #[cfg(target_family = "windows")]
    {
        let mut old_prot: winapi::shared::minwindef::DWORD = 0;
        VirtualProtect(
            pageaddr as *mut _,
            *PAGESIZE as _,
            PAGE_NOACCESS,
            &mut old_prot,
        );
    }

    #[cfg(target_family = "unix")]
    {
        mprotect(pageaddr.cast(), *PAGESIZE, PROT_NONE);
    }
}
unsafe fn disable_safepoint(_idx: usize) {
    /*SAFEPOINT_ENABLE_CNT -= 1;
    if SAFEPOINT_ENABLE_CNT != 0 {
        assert!(SAFEPOINT_ENABLE_CNT > 0);
        return;
    }*/
    let pageaddr = SAFEPOINT_PAGE;
    #[cfg(target_family = "windows")]
    {
        let mut old_prot: winapi::shared::minwindef::DWORD = 0;
        VirtualProtect(
            pageaddr as *mut _,
            *PAGESIZE as _,
            PAGE_READONLY,
            &mut old_prot,
        );
    }

    #[cfg(target_family = "unix")]
    {
        mprotect(pageaddr.cast(), *PAGESIZE, PROT_READ);
    }
}
#[allow(unused_mut)]
pub unsafe fn safepoint_init() {
    let pgsz = *PAGESIZE;

    let mut addr;
    #[cfg(target_family = "unix")]
    {
        addr = mmap(
            0 as *mut _,
            pgsz,
            PROT_READ,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        );
        if addr == MAP_FAILED {
            addr = std::ptr::null_mut();
        }
    };
    #[cfg(target_family = "windows")]
    {
        addr = VirtualAlloc(0 as *mut _, pgsz, MEM_COMMIT, PAGE_READONLY);
    }

    if addr.is_null() {
        panic!("could not allocate GC synchronization page");
    }

    SAFEPOINT_PAGE = addr.cast();
}

pub fn safepoint_start_gc() -> bool {
    //assert!(get_tls_state().gc_state == GC_STATE_WAITING);
    unsafe {
        SAFEPOINT_LOCK.lock_nogc();

        if GC_RUNNING.compare_exchange_weak(false, true, Ordering::SeqCst, Ordering::Relaxed)
            != Ok(false)
        {
            // if other thread started GC first we suspend current thread and allow other thread to run GC cycle.
            SAFEPOINT_LOCK.unlock_nogc();
            safepoint_wait_gc();
            return false;
        }

        enable_safepoint();
        SAFEPOINT_LOCK.unlock_nogc();
    }
    true
}

pub fn safepoint_wait_for_the_world() -> parking_lot::MutexGuard<'static, Vec<*mut TLSState>> {
    let threads = &*THREADS;
    let ctls = get_tls_state() as *mut _;
    //panic!();
    let lock = threads.threads.lock();

    for th in lock.iter() {
        if *th == ctls {
            continue;
        }

        let ptls = unsafe { &**th };
        //println!("wait on {:p}", ptls);
        while ptls.atomic_gc_state().load(Ordering::Relaxed) == 0
            || ptls.atomic_gc_state().load(Ordering::Acquire) == 0
        {
            std::sync::atomic::spin_loop_hint();
        }
    }
    lock
}

pub fn safepoint_end_gc() {
    unsafe {
        SAFEPOINT_LOCK.lock_nogc();

        disable_safepoint(1);
        disable_safepoint(2);
        GC_RUNNING.store(false, Ordering::Release);
        SAFEPOINT_LOCK.unlock_nogc();
    }
}

pub fn safepoint_wait_gc() {
    while GC_RUNNING.load(Ordering::Relaxed) || GC_RUNNING.load(Ordering::Acquire) {
        std::sync::atomic::spin_loop_hint();
    }
}

pub fn addr_in_safepoint(addr: usize) -> bool {
    unsafe {
        let safepoint_addr = SAFEPOINT_PAGE as usize;
        addr == safepoint_addr
    }
}
