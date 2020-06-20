use std::sync::Arc;
pub mod fullcodegen;
pub mod function;
pub mod gc;
pub mod module;
pub mod object;
pub mod pure_nan;
pub mod runtime;
pub mod tagged;
pub mod thread;
pub mod value;
pub mod vtable;
pub const WORD: usize = std::mem::size_of::<usize>();

pub struct VirtualMachine {
    pub state: Arc<GlobalState>,
}

impl VirtualMachine {
    pub fn collect(&self) {
        let x = false;
        thread::stop_the_world(
            |threads| {
                self.state.heap.collect(threads);
            },
            &x,
        );
    }

    pub fn register_thread(&self, top: *const bool) {
        /*gc::immix_space::LOCAL_ALLOCATOR.with(|l| {
            (&*l.get()).stack_top.store(top as usize);
        });*/
        self.state.threads.attach_current_thread(top as *const u8);
    }
}

pub struct GlobalState {
    pub threads: thread::Threads,
    pub heap: gc::Heap,
}

lazy_static::lazy_static! {
    pub static ref VM: VirtualMachine = VirtualMachine {
        state: Arc::new(GlobalState {
            threads: thread::Threads::new(),
            heap: gc::Heap::new()
        })
    };
}
