/*
*   Copyright (c) 2020 Adel Prokurov
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/
use super::*;
use crate::util::arc::Arc;
use state::*;
use threads::*;

pub fn stop_the_world<F, R>(vm: &Runtime, f: F) -> R
where
    F: FnOnce(&[Arc<WaffleThread>]) -> R,
{
    THREAD.with(|thread| thread.borrow().park());

    let threads = vm.state.threads.threads.lock();
    if threads.len() == 1 {
        let ret = f(&*threads);
        THREAD.with(|thread| thread.borrow().unpark());
        return ret;
    }

    let safepoint_id = stop_threads(vm, &*threads);
    let ret = f(&*threads);
    resume_threads(vm, &*threads, safepoint_id);
    THREAD.with(|thread| thread.borrow().unpark());
    ret
}

fn current_thread_id() -> usize {
    THREAD.with(|thread| thread.borrow().id())
}

fn stop_threads(vm: &Runtime, threads: &[Arc<WaffleThread>]) -> usize {
    let thread_self = THREAD.with(|thread| thread.borrow().clone());
    let safepoint_id = vm.state.threads.request_safepoint();

    vm.state.threads.barrier.guard(safepoint_id);

    for thread in threads.iter() {
        thread.arm_stack_guard();
    }

    while !all_threads_blocked(vm, &thread_self, threads, safepoint_id) {
        // do nothing
    }

    safepoint_id
}

fn all_threads_blocked(
    _vm: &Runtime,
    thread_self: &Arc<WaffleThread>,
    threads: &[Arc<WaffleThread>],
    safepoint_id: usize,
) -> bool {
    let mut all_blocked = true;

    for thread in threads {
        if Arc::ptr_eq(thread, thread_self) {
            assert!(thread.state().is_parked());
            continue;
        }

        if !thread.in_safepoint(safepoint_id) {
            all_blocked = false;
        }
    }

    all_blocked
}

fn resume_threads(rt: &Runtime, threads: &[Arc<WaffleThread>], safepoint_id: usize) {
    for thread in threads.iter() {
        thread.unarm_stack_guard();
    }

    rt.state.threads.barrier.resume(safepoint_id);
    rt.state.threads.clear_safepoint_request();
}

pub extern "C" fn guard_check() {
    let thread = THREAD.with(|thread| thread.borrow().clone());
    let stack_overflow = thread.real_stack_limit() > stack_pointer().to_usize();

    if stack_overflow {
        panic!("Stack overflow");
    } else {
        block(&RUNTIME, &thread);
    }
}

pub fn block(rt: &Runtime, thread: &WaffleThread) {
    let safepoint_id = rt.state.threads.safepoint_id();
    assert_ne!(safepoint_id, 0);
    let state = thread.state();

    match state {
        ThreadState::Running | ThreadState::Parked => {
            thread.block(safepoint_id);
        }
        ThreadState::Blocked => {
            panic!("illegal thread state: thread #{} {:?}", thread.id(), state);
        }
    };

    let _mtx = rt.state.threads.barrier.wait(safepoint_id);
    thread.unblock();
}
