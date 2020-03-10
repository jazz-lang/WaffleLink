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

use super::cell::*;
use super::channel::Channel;
use super::state::*;
use super::value::*;
use super::*;
use crate::heap::{initialize_process_heap, GCType, HeapTrait};
use crate::interpreter::context::*;
use crate::util;

use parking_lot::{Condvar, Mutex};
use std::cell::RefCell;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use util::arc::*;
use util::ptr::*;
use util::tagged;
use util::tagged::*;
thread_local! {
    pub static THREAD: RefCell<Arc<WaffleThread>> = RefCell::new(WaffleThread::new());
}

pub struct Threads {
    pub threads: Mutex<Vec<Arc<WaffleThread>>>,
    pub cond_join: Condvar,

    pub next_id: AtomicUsize,
    pub safepoint: Mutex<(usize, usize)>,

    pub barrier: Barrier,
}

impl Threads {
    pub fn new() -> Threads {
        Threads {
            threads: Mutex::new(Vec::new()),
            cond_join: Condvar::new(),
            next_id: AtomicUsize::new(1),
            safepoint: Mutex::new((0, 1)),
            barrier: Barrier::new(),
        }
    }

    pub fn attach_current_thread(&self) {
        THREAD.with(|thread| {
            let mut threads = self.threads.lock();
            threads.push(thread.borrow().clone());
        });
    }

    pub fn attach_thread(&self, thread: Arc<WaffleThread>) {
        let mut threads = self.threads.lock();
        threads.push(thread);
    }

    pub fn next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn safepoint_id(&self) -> usize {
        let safepoint = self.safepoint.lock();
        safepoint.0
    }

    pub fn safepoint_requested(&self) -> bool {
        let safepoint = self.safepoint.lock();
        safepoint.0 != 0
    }

    pub fn request_safepoint(&self) -> usize {
        let mut safepoint = self.safepoint.lock();
        assert_eq!(safepoint.0, 0);
        safepoint.0 = safepoint.1;
        safepoint.1 += 1;

        safepoint.0
    }

    pub fn clear_safepoint_request(&self) {
        let mut safepoint = self.safepoint.lock();
        assert_ne!(safepoint.0, 0);
        safepoint.0 = 0;
    }

    pub fn detach_current_thread(&self) {
        let state = &RUNTIME.state;
        //let vm = get_vm();

        // Other threads might still be running and perform a GC.
        // Fill the TLAB for them.
        //tlab::make_iterable_current(vm);

        THREAD.with(|thread| {
            thread.borrow_mut().park();
            let mut threads = self.threads.lock();
            threads.retain(|elem| !Arc::ptr_eq(elem, &*thread.borrow()));
            self.cond_join.notify_all();
        });
    }

    pub fn join_all(&self) {
        let mut threads = self.threads.lock();

        while threads.len() > 0 {
            self.cond_join.wait(&mut threads);
        }
    }

    pub fn each<F>(&self, mut f: F)
    where
        F: FnMut(&Arc<WaffleThread>),
    {
        let threads = self.threads.lock();

        for thread in threads.iter() {
            f(thread)
        }
    }
}
pub struct CatchTable {
    pub jump_to: u16,
    pub context: Ptr<Context>,
    pub register: u16,
}

pub struct LocalData {
    pub context: Ptr<Context>,
    pub catch_tables: Vec<CatchTable>,
    pub status: WaffleThreadStatus,
    pub thread_id: Option<u8>,
    pub state: StateManager,
}

/// Lightweight "green" process.
///
/// This sturcture represents lightweight process. Each process scheduled by the virtual machine.
pub struct WaffleThread {
    pub local_data: Ptr<LocalData>,
}

impl WaffleThread {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            local_data: Ptr::new(LocalData {
                catch_tables: vec![],
                context: Ptr::null(),
                status: WaffleThreadStatus::new(),
                thread_id: None,
                state: StateManager::new(),
            }),
        })
    }

    pub fn with_rc(context: Context, config: &super::config::Config) -> Arc<Self> {
        let local_data = LocalData {
            catch_tables: vec![],
            context: Ptr::new(context),
            status: WaffleThreadStatus::new(),
            thread_id: None,
            state: StateManager::new(),
        };
        let proc = Arc::new(Self {
            local_data: Ptr::new(local_data),
        });
        proc
    }

    pub fn from_function(
        value: Value,
        config: &super::config::Config,
    ) -> Result<Arc<Self>, String> {
        if value.is_cell() == false {
            return Err(format!("Expected function to WaffleThread.spawn",).to_owned());
        };

        let value = value.as_cell();
        match value.get().value {
            CellValue::Function(ref function) => {
                let context = Context {
                    arguments: vec![],
                    bindex: 0,
                    in_tail: false,
                    index: 0,
                    code: function.code.clone(),
                    function: Value::from(value),
                    parent: None,
                    module: function.module.clone(),
                    registers: [Value::from(VTag::Undefined); 32],
                    stack: vec![],
                    this: Value::from(VTag::Undefined),
                    return_register: None,
                    terminate_upon_return: true,
                    n: 0,
                };

                Ok(Self::with_rc(context, config))
            }
            _ => return Err(format!("Expected function to WaffleThread.spawn").to_owned()),
        }
    }

    pub fn local_data(&self) -> &LocalData {
        self.local_data.get()
    }

    pub fn local_data_mut(&self) -> &mut LocalData {
        self.local_data.get()
    }
    pub fn is_pinned(&self) -> bool {
        self.thread_id().is_some()
    }

    pub fn terminate(&self, _: &State) {
        // Once terminated we don't want to receive any messages any more, as
        // they will never be received and thus lead to an increase in memory.
        // Thus, we mark the process as terminated. We must do this _after_
        // acquiring the lock to ensure other processes sending messages will
        // observe the right value.
        self.set_terminated();
    }
    pub fn pop_context(&self) -> bool {
        let local_data = self.local_data_mut();
        if let Some(parent) = local_data.context.parent.take() {
            let old = local_data.context;
            if old.raw.is_null() {
                return true;
            }
            unsafe {
                //std::ptr::drop_in_place(old.raw);
                let _ = Box::from_raw(old.raw);
            }
            local_data.context = parent;

            false
        } else {
            true
        }
    }
    pub fn pop_context_not_drop(&self) -> bool {
        let local_data = self.local_data_mut();
        if let Some(parent) = local_data.context.parent.take() {
            let old = local_data.context;
            if old.raw.is_null() {
                return true;
            }
            local_data.context = parent;

            false
        } else {
            true
        }
    }
    pub fn push_context_ptr(&self, mut context: Ptr<Context>) {
        let local_data = self.local_data_mut();
        let target = &mut local_data.context;
        std::mem::swap(target, &mut context);
        target.parent = Some(context);
    }

    pub fn push_context(&self, context: Context) {
        let mut boxed = Ptr::new(context);
        let local_data = self.local_data_mut();
        let target = &mut local_data.context;

        std::mem::swap(target, &mut boxed);

        target.parent = Some(boxed);
    }

    pub fn context_mut(&self) -> &mut Context {
        self.local_data_mut().context.get()
    }

    pub fn context_ptr(&self) -> Ptr<Context> {
        self.local_data_mut().context
    }

    pub fn context(&self) -> &Context {
        self.local_data().context.get()
    }

    pub fn trace<F>(&self, mut cb: F)
    where
        F: FnMut(*const super::cell::CellPointer),
    {
        let ld = self.local_data();
        let ctx = &ld.context;
        if ctx.raw.is_null() {
            return;
        }
        let mut current = Some(&**ctx);
        while let Some(context) = current {
            context.registers.iter().for_each(|x| {
                if x.is_cell() {
                    unsafe { cb(&x.u.ptr) }
                }
            });

            context.stack.iter().for_each(|x| {
                if x.is_cell() {
                    unsafe { cb(&x.u.ptr) }
                }
            });
            context.module.globals.iter().for_each(|x| {
                if x.is_cell() {
                    unsafe { cb(&x.u.ptr) }
                }
            });
            cb(&context.function.as_cell());
            current = context.parent.as_ref().map(|c| &**c);
        }
    }
    pub fn is_terminated(&self) -> bool {
        self.local_data().status.is_terminated()
    }
    pub fn set_terminated(&self) {
        self.local_data_mut().status.set_terminated();
    }

    pub fn thread_id(&self) -> Option<u8> {
        self.local_data().thread_id
    }
    pub fn set_thread_id(&self, id: u8) {
        self.local_data_mut().thread_id = Some(id);
    }

    pub fn unset_thread_id(&self) {
        self.local_data_mut().thread_id = None;
    }
    pub fn set_main(&self) {
        self.local_data_mut().status.set_main();
    }

    pub fn is_main(&self) -> bool {
        self.local_data().status.is_main()
    }

    pub fn park(&self) {
        self.local_data_mut().state.park()
    }
    pub fn unpark(&self) {
        if RUNTIME.state.threads.safepoint_id() != 0 {}
        self.local_data_mut().state.unpark();
    }

    pub fn block(&self, id: usize) {
        self.local_data_mut().state.block(id);
    }
    pub fn in_safepoint(&self, id: usize) -> bool {
        self.local_data().state.in_safepoint(id)
    }
    pub fn do_gc(this: &Arc<WaffleThread>) {
        let local_data = this.local_data_mut();
        //let _ = local_data.heap.collect_garbage(this);
    }

    pub fn gc_enable(&self) {
        let local_data = self.local_data_mut();
        //local_data.heap.enable();
    }

    pub fn gc_disable(&self) {
        let local_data = self.local_data_mut();
        //local_data.heap.disable();
    }

    pub fn gc_is_enabled(&self) -> bool {
        //self.local_data().heap.is_enabled()
        true
    }

    pub fn allocate_string(this: &Arc<WaffleThread>, state: &RcState, string: &str) -> Value {
        let local_data = this.local_data_mut();
        /*let cell = local_data.heap.allocate(
            this,
            GCType::Young,
            Cell::with_prototype(
                CellValue::String(Arc::new(String::from(string))),
                state.string_prototype.as_cell(),
            ),
        );
        Value::from(cell)*/
        unimplemented!()
    }

    pub fn allocate(this: &Arc<WaffleThread>, cell: Cell) -> Value {
        let local_data = this.local_data_mut();
        //let cell = local_data.heap.allocate(this, GCType::Young, cell);
        //Value::from(cell)
        unimplemented!()
    }
}

impl PartialEq for Arc<WaffleThread> {
    fn eq(&self, other: &Arc<WaffleThread>) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

/// The status of a process, represented as a set of bits.
///
/// We use an atomic U8 since an external process may read this value while we
/// are changing it (e.g. when a process sends a message while the receiver
/// enters the blocking status).
///
/// While concurrent reads are allowed, only the owning process should change
/// the status.
pub struct WaffleThreadStatus {
    /// The bits used to indicate the status of the process.
    ///
    /// Multiple bits may be set in order to combine different statuses. For
    /// example, if the main process is blocking it will set both bits.
    bits: AtomicU8,
}

impl WaffleThreadStatus {
    /// A regular thread.
    const NORMAL: u8 = 0b0;

    /// The main thread.
    const MAIN: u8 = 0b1;

    /// The thread is suspended at safepoint.
    const SAFEPOINT_SUSPEND: u8 = 0b10;
    const SUSPENDED: u8 = 0b1000;
    /// The process is terminated.
    const TERMINATED: u8 = 0b100;

    pub fn new() -> Self {
        Self {
            bits: AtomicU8::new(Self::NORMAL),
        }
    }

    pub fn set_main(&mut self) {
        self.update_bits(Self::MAIN, true);
    }

    pub fn is_main(&self) -> bool {
        self.bit_is_set(Self::MAIN)
    }

    pub fn set_terminated(&mut self) {
        self.update_bits(Self::TERMINATED, true);
    }

    pub fn is_terminated(&self) -> bool {
        self.bit_is_set(Self::TERMINATED)
    }

    fn update_bits(&mut self, mask: u8, enable: bool) {
        let bits = self.bits.load(Ordering::Acquire);
        let new_bits = if enable { bits | mask } else { bits & !mask };

        self.bits.store(new_bits, Ordering::Release);
    }

    fn bit_is_set(&self, bit: u8) -> bool {
        self.bits.load(Ordering::Acquire) & bit == bit
    }
}

impl Drop for WaffleThread {
    fn drop(&mut self) {
        unsafe {
            while !self.pop_context() {}
            std::ptr::drop_in_place(self.local_data.raw);
        }
    }
}

pub struct Barrier {
    active: Mutex<usize>,
    done: Condvar,
}

impl Barrier {
    pub fn new() -> Barrier {
        Barrier {
            active: Mutex::new(0),
            done: Condvar::new(),
        }
    }

    pub fn guard(&self, safepoint_id: usize) {
        let mut active = self.active.lock();
        assert_eq!(*active, 0);
        assert_ne!(safepoint_id, 0);
        *active = safepoint_id;
    }

    pub fn resume(&self, safepoint_id: usize) {
        let mut active = self.active.lock();
        assert_eq!(*active, safepoint_id);
        assert_ne!(safepoint_id, 0);
        *active = 0;
        self.done.notify_all();
    }

    pub fn wait(&self, safepoint_id: usize) {
        let mut active = self.active.lock();
        assert_ne!(safepoint_id, 0);

        while *active == safepoint_id {
            self.done.wait(&mut active);
        }
    }
}

pub struct StateManager {
    mtx: Mutex<(ThreadState, usize)>,
}

impl StateManager {
    fn new() -> StateManager {
        StateManager {
            mtx: Mutex::new((ThreadState::Running, 0)),
        }
    }

    fn state(&self) -> ThreadState {
        let mtx = self.mtx.lock();
        mtx.0
    }

    fn park(&self) {
        let mut mtx = self.mtx.lock();
        assert!(mtx.0.is_running());
        mtx.0 = ThreadState::Parked;
    }

    fn unpark(&self) {
        let mut mtx = self.mtx.lock();
        assert!(mtx.0.is_parked());
        mtx.0 = ThreadState::Running;
    }

    fn block(&self, safepoint_id: usize) {
        let mut mtx = self.mtx.lock();
        assert!(mtx.0.is_running());
        mtx.0 = ThreadState::Blocked;
        mtx.1 = safepoint_id;
    }

    fn unblock(&self) {
        let mut mtx = self.mtx.lock();
        assert!(mtx.0.is_blocked());
        mtx.0 = ThreadState::Running;
        mtx.1 = 0;
    }

    fn in_safepoint(&self, safepoint_id: usize) -> bool {
        assert_ne!(safepoint_id, 0);
        let mtx = self.mtx.lock();

        match mtx.0 {
            ThreadState::Running => false,
            ThreadState::Blocked => mtx.1 == safepoint_id,
            ThreadState::Parked => true,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ThreadState {
    Running = 0,
    Parked = 1,
    Blocked = 2,
}

impl From<usize> for ThreadState {
    fn from(value: usize) -> ThreadState {
        match value {
            0 => ThreadState::Running,
            1 => ThreadState::Parked,
            2 => ThreadState::Blocked,
            _ => unreachable!(),
        }
    }
}

impl ThreadState {
    pub fn is_running(&self) -> bool {
        match *self {
            ThreadState::Running => true,
            _ => false,
        }
    }

    pub fn is_parked(&self) -> bool {
        match *self {
            ThreadState::Parked => true,
            _ => false,
        }
    }

    pub fn is_blocked(&self) -> bool {
        match *self {
            ThreadState::Blocked => true,
            _ => false,
        }
    }

    pub fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl Default for ThreadState {
    fn default() -> ThreadState {
        ThreadState::Running
    }
}
