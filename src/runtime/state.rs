use super::cell::*;
use super::interner::*;
use super::scheduler;
use super::value::*;
use crate::heap::PermanentHeap;
use crate::util::arc::Arc;
use parking_lot::Mutex;
use scheduler::process_scheduler::ProcessScheduler;
use scheduler::timeout_worker::TimeoutWorker;
pub struct State {
    pub scheduler: ProcessScheduler,
    pub timeout_worker: TimeoutWorker,
    pub perm_heap: Mutex<PermanentHeap>,
    pub string_prototype: Value,
    pub object_prototype: Value,
    pub array_prototype: Value,
    pub number_prototype: Value,
    pub function_prototype: Value,
    pub generator_prototype: Value,
    pub process_prototype: Value,
    pub module_prototype: Value,
    pub boolean_prototype: Value,
    pub byte_array_prototype: Value,
    pub static_variables: ahash::AHashMap<String, Value>,
    pub config: super::config::Config,
    pub string_pool: Mutex<StringPool>,
}

macro_rules! intern_string {
    ($state:expr, $lookup:expr, $store:expr) => {{
        let mut pool = $state.string_pool.lock();

        if let Some(value) = pool.get($lookup) {
            return value;
        }

        let ptr = {
            let mut alloc = $state.perm_heap.lock();
            let value = CellValue::InternedString($store);

            alloc.allocate_with_prototype(value, $state.string_prototype.as_cell())
        };

        pool.add(ptr);

        ptr
    }};
}

#[inline]
fn nof_parallel_worker_threads(num: usize, den: usize, switch_pt: usize) -> usize {
    let ncpus = num_cpus::get();
    if ncpus <= switch_pt {
        if ncpus <= 1 {
            return 2;
        }
        ncpus
    } else {
        switch_pt + ((ncpus - switch_pt) * num) / den
    }
}

impl State {
    pub fn new(config: super::config::Config) -> Arc<Self> {
        let mut perm = PermanentHeap::new(config.perm_size);
        let object_prototype = perm.allocate_empty();
        let string_prototype = perm.allocate_empty();
        let array_prototype = perm.allocate_empty();
        let number_prototype = perm.allocate_empty();
        let function_prototype = perm.allocate_empty();
        let generator_prototype = perm.allocate_empty();
        let process_prototype = perm.allocate_empty();
        let module_prototype = perm.allocate_empty();
        let boolean_prototype = perm.allocate_empty();
        let byte_array_prototype = perm.allocate_empty();
        string_prototype.set_prototype(object_prototype);
        array_prototype.set_prototype(object_prototype);
        number_prototype.set_prototype(object_prototype);
        function_prototype.set_prototype(object_prototype);
        generator_prototype.set_prototype(object_prototype);
        process_prototype.set_prototype(object_prototype);
        module_prototype.set_prototype(object_prototype);
        boolean_prototype.set_prototype(object_prototype);
        byte_array_prototype.set_prototype(object_prototype);
        let primary = if let Some(prim) = config.primary {
            prim
        } else {
            nof_parallel_worker_threads(5, 8, 8)
        };
        let blocking = if let Some(blocking) = config.blocking {
            blocking
        } else {
            nof_parallel_worker_threads(5, 8, 8)
        };
        let scheduler = ProcessScheduler::new(primary, blocking);
        let timeout_worker = TimeoutWorker::new();
        let _gc_workers = if let Some(c) = config.gc_workers {
            c
        } else {
            nof_parallel_worker_threads(5, 8, 8) / 2
        };
        Arc::new(Self {
            scheduler,
            timeout_worker,
            perm_heap: Mutex::new(perm),
            object_prototype,
            string_prototype,
            array_prototype,
            number_prototype,
            boolean_prototype,
            byte_array_prototype,
            process_prototype,
            module_prototype,
            function_prototype,
            generator_prototype,
            static_variables: ahash::AHashMap::new(),
            config,
            string_pool: Mutex::new(StringPool::new()),
        })
    }

    pub fn allocate(&self, cell: Cell) -> Value {
        Value::from(self.perm_heap.lock().allocate(cell))
    }

    pub fn allocate_fn(&self, fun: Function) -> Value {
        Value::from(self.perm_heap.lock().allocate(Cell::with_prototype(
            CellValue::Function(Arc::new(fun)),
            self.function_prototype.as_cell(),
        )))
    }
    pub fn allocate_native_fn(
        &self,
        native_fn: super::cell::NativeFn,
        name: &str,
        argc: i32,
    ) -> CellPointer {
        let function = Function {
            name: Value::from(self.intern_string(name.to_owned())),
            upvalues: vec![],
            code: Arc::new(vec![]),
            native: Some(native_fn),
            argc,
            module: Arc::new(super::module::Module {
                globals: vec![],
                name: Arc::new("<native>".to_owned()),
                main_fn: Value::empty(),
            }),
            md: Default::default(),
        };

        let cell = self
            .perm_heap
            .lock()
            .allocate(Cell::with_prototype(
                CellValue::Function(Arc::new(function)),
                self.function_prototype.as_cell(),
            ))
            .as_cell();
        cell
    }
    pub fn allocate_native_fn_with_name(
        &self,
        native_fn: super::cell::NativeFn,
        name: &str,
        argc: i32,
    ) -> CellPointer {
        let function = Function {
            name: Value::from(self.intern_string(name.to_owned())),
            upvalues: vec![],
            code: Arc::new(vec![]),
            native: Some(native_fn),
            argc,
            module: Arc::new(super::module::Module {
                globals: vec![],
                name: Arc::new("<native>".to_owned()),
                main_fn: Value::empty(),
            }),
            md: Default::default(),
        };

        let cell = self
            .perm_heap
            .lock()
            .allocate(Cell::with_prototype(
                CellValue::Function(Arc::new(function)),
                self.function_prototype.as_cell(),
            ))
            .as_cell();
        cell
    }

    /// Interns a borrowed String.
    ///
    /// If a string was not yet interned it's allocated in the permanent space.
    pub fn intern(&self, string: &String) -> CellPointer {
        intern_string!(self, string, Arc::new(string.clone()))
    }

    /// Interns an owned String.
    pub fn intern_string(&self, string: String) -> CellPointer {
        let to_intern = string;

        intern_string!(self, &to_intern, Arc::new(to_intern))
    }

    /// Interns a pointer pointing to a string.
    ///
    /// If the pointer is already interned it's simply returned.
    pub fn intern_pointer(&self, pointer: CellPointer) -> CellPointer {
        if pointer.is_interned_str() {
            pointer
        } else {
            self.intern(&pointer.to_string())
        }
    }
}

pub type RcState = Arc<State>;
