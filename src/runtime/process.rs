use super::cell::*;
use super::channel::Channel;
use super::scheduler::timeout::*;
use super::state::*;
use super::value::*;
use crate::heap::{initialize_process_heap, GCType, HeapTrait};
use crate::interpreter::context::*;
use crate::util;

use parking_lot::Mutex;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use util::arc::*;
use util::ptr::*;
use util::tagged;
use util::tagged::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug)]
#[repr(u8)]
pub enum ProcessPriority {
    Max,
    High,
    Normal,
    Low,
}

/// The bit that is set to mark a process as being suspended.
const SUSPENDED_BIT: usize = 0;

/// An enum describing what rights a thread was given when trying to reschedule
/// a process.
pub enum RescheduleRights {
    /// The rescheduling rights were not obtained.
    Failed,

    /// The rescheduling rights were obtained.
    Acquired,

    /// The rescheduling rights were obtained, and the process was using a
    /// timeout.
    AcquiredWithTimeout(Arc<Timeout>),
}

impl RescheduleRights {
    pub fn are_acquired(&self) -> bool {
        match self {
            RescheduleRights::Failed => false,
            _ => true,
        }
    }

    pub fn process_had_timeout(&self) -> bool {
        match self {
            RescheduleRights::AcquiredWithTimeout(_) => true,
            _ => false,
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
    /// Channel of this process. This channel is used like `std::sync::mpsc` channel.
    pub channel: Mutex<Channel>,
    pub status: ProcessStatus,
    pub heap: Box<dyn HeapTrait>,
    pub thread_id: Option<u8>,
}

/// Lightweight "green" process.
///
/// This sturcture represents lightweight process. Each process scheduled by the virtual machine.
pub struct Process {
    pub local_data: Ptr<LocalData>,
    /// If the process is waiting for a message.
    waiting_for_message: AtomicBool,

    /// A marker indicating if a process is suspened, optionally including the
    /// pointer to the timeout.
    ///
    /// When this value is NULL, the process is not suspended.
    ///
    /// When the lowest bit is set to 1, the pointer may point to (after
    /// unsetting the bit) to one of the following:
    ///
    /// 1. NULL, meaning the process is suspended indefinitely.
    /// 2. A Timeout, meaning the process is suspended until the timeout
    ///    expires.
    ///
    /// While the type here uses a `TaggedPointer`, in reality the type is an
    /// `Arc<Timeout>`. This trick is needed to allow for atomic
    /// operations and tagging, something which isn't possible using an
    /// `Option<T>`.
    suspended: TaggedPointer<Timeout>,
    pub priority: AtomicU8,
}

impl Process {
    pub fn with_rc(context: Context, config: &super::config::Config) -> Arc<Self> {
        let local_data = LocalData {
            heap: initialize_process_heap(config.gc, config),
            channel: Mutex::new(Channel::new()),
            catch_tables: vec![],
            context: Ptr::new(context),
            status: ProcessStatus::new(),
            thread_id: None,
        };
        assert!(local_data.context.raw.is_null() == false);
        let proc = Arc::new(Self {
            waiting_for_message: AtomicBool::new(false),
            suspended: TaggedPointer::null(),
            local_data: Ptr::new(local_data),
            priority: AtomicU8::new(ProcessPriority::Normal as u8),
        });

        proc.local_data_mut().heap.set_proc(proc.clone());
        proc
    }

    pub fn from_function(
        value: Value,
        config: &super::config::Config,
    ) -> Result<Arc<Self>, String> {
        if value.is_cell() == false {
            return Err("Expected function to Process.spawn".to_owned());
        };

        let value = value.as_cell();
        match value.get().value {
            CellValue::Function(ref function) => {
                let context = Context {
                    bindex: 0,
                    index: 0,
                    code: function.code.clone(),
                    function: value,
                    parent: None,
                    module: function.module.clone(),
                    registers: [Value::from(VTag::Undefined); 48],
                    stack: vec![],
                    this: Value::from(VTag::Undefined),
                    return_register: None,
                    terminate_upon_return: true,
                    n: 0,
                };

                Ok(Self::with_rc(context, config))
            }
            _ => return Err("Expected function to Process.spawn".to_owned()),
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
        // The channel lock _must_ be acquired first, otherwise we may end up
        // reclaiming memory while another process is allocating message into
        // them.
        let _channel = self.local_data().channel.lock();
        // Once terminated we don't want to receive any messages any more, as
        // they will never be received and thus lead to an increase in memory.
        // Thus, we mark the process as terminated. We must do this _after_
        // acquiring the lock to ensure other processes sending messages will
        // observe the right value.
        self.set_terminated();

        self.local_data_mut().heap.clear();
    }
    pub fn pop_context(&self) -> bool {
        let local_data = self.local_data_mut();
        if let Some(parent) = local_data.context.parent.take() {
            let old = local_data.context;
            if old.raw.is_null() {
                return true;
            }
            unsafe {
                std::ptr::drop_in_place(old.raw);
            }
            local_data.context = parent;

            false
        } else {
            true
        }
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
                if x.is_cell() && !x.is_null_or_undefined() && !x.is_empty() {
                    unsafe { cb(&x.u.ptr) }
                }
            });

            context.stack.iter().for_each(|x| {
                if x.is_cell() {
                    unsafe {
                        if x.u.as_int64 as u64 == 0xfffe00000000002a {
                            panic!();
                        }
                    }
                    unsafe { cb(&x.u.ptr) }
                }
            });
            context.module.globals.iter().for_each(|x| {
                if x.is_cell() {
                    unsafe { cb(&x.u.ptr) }
                }
            });
            cb(&context.function);
            current = context.parent.as_ref().map(|c| &**c);
        }
    }

    pub fn suspend_with_timeout(&self, timeout: Arc<Timeout>) {
        let pointer = Arc::into_raw(timeout);
        let tagged = tagged::with_bit(pointer, SUSPENDED_BIT);

        self.suspended.atomic_store(tagged);
    }

    pub fn suspend_without_timeout(&self) {
        let pointer = ptr::null_mut();
        let tagged = tagged::with_bit(pointer, SUSPENDED_BIT);

        self.suspended.atomic_store(tagged);
    }

    pub fn is_suspended_with_timeout(&self, timeout: &Arc<Timeout>) -> bool {
        let pointer = self.suspended.atomic_load();

        tagged::untagged(pointer) == timeout.as_ptr()
    }

    /// Attempts to acquire the rights to reschedule this process.
    pub fn acquire_rescheduling_rights(&self) -> RescheduleRights {
        let current = self.suspended.atomic_load();

        if current.is_null() {
            RescheduleRights::Failed
        } else if self.suspended.compare_and_swap(current, ptr::null_mut()) {
            let untagged = tagged::untagged(current);

            if untagged.is_null() {
                RescheduleRights::Acquired
            } else {
                let timeout = unsafe { Arc::from_raw(untagged) };

                RescheduleRights::AcquiredWithTimeout(timeout)
            }
        } else {
            RescheduleRights::Failed
        }
    }

    pub fn send_message_from_external_process(&self, message_to_copy: Value) {
        let local_data = self.local_data_mut();

        // The lock must be acquired first, as the receiving process may be
        // garbage collected at this time.
        let mut channel = local_data.channel.lock();

        // When a process terminates it will acquire the channel lock first.
        // Checking the status after acquiring the lock allows us to obtain a
        // stable view of the status.
        if self.is_terminated() {
            return;
        }

        channel.send(local_data.heap.copy_object(message_to_copy));
    }

    pub fn send_message_from_self(&self, message: Value) {
        self.local_data_mut().channel.lock().send(message);
    }

    pub fn waiting_for_message(&self) {
        self.waiting_for_message.store(true, Ordering::Release);
    }

    pub fn no_longer_waiting_for_message(&self) {
        self.waiting_for_message.store(false, Ordering::Release);
    }

    pub fn is_waiting_for_message(&self) -> bool {
        self.waiting_for_message.load(Ordering::Acquire)
    }

    pub fn receive_message(&self) -> Option<Value> {
        self.local_data_mut().channel.lock().receive()
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

    pub fn set_blocking(&self, enable: bool) {
        self.local_data_mut().status.set_blocking(enable);
    }

    pub fn is_blocking(&self) -> bool {
        self.local_data().status.is_blocking()
    }

    pub fn do_gc(this: &Arc<Process>) {
        let local_data = this.local_data_mut();
        local_data.heap.collect_garbage(this);
    }

    pub fn allocate_string(&self, state: &RcState, string: &str) -> Value {
        let local_data = self.local_data_mut();
        let cell = local_data.heap.allocate(
            GCType::Young,
            Cell::with_prototype(
                CellValue::String(Arc::new(String::from(string))),
                state.string_prototype.as_cell(),
            ),
        );
        Value::from(cell)
    }

    pub fn has_messages(&self) -> bool {
        self.local_data().channel.lock().has_messages()
    }

    pub fn allocate(&self, cell: Cell) -> Value {
        let local_data = self.local_data_mut();
        let cell = local_data.heap.allocate(GCType::Young, cell);
        Value::from(cell)
    }
}

impl PartialEq for Arc<Process> {
    fn eq(&self, other: &Arc<Process>) -> bool {
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
pub struct ProcessStatus {
    /// The bits used to indicate the status of the process.
    ///
    /// Multiple bits may be set in order to combine different statuses. For
    /// example, if the main process is blocking it will set both bits.
    bits: AtomicU8,
}

impl ProcessStatus {
    /// A regular process.
    const NORMAL: u8 = 0b0;

    /// The main process.
    const MAIN: u8 = 0b1;

    /// The process is performing a blocking operation.
    const BLOCKING: u8 = 0b10;

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

    pub fn set_blocking(&mut self, enable: bool) {
        self.update_bits(Self::BLOCKING, enable);
    }

    pub fn is_blocking(&self) -> bool {
        self.bit_is_set(Self::BLOCKING)
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

impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            while !self.pop_context() {}
            std::ptr::drop_in_place(self.local_data.raw);
            std::ptr::drop_in_place(self.suspended.raw);
        }
        self.acquire_rescheduling_rights();
    }
}
