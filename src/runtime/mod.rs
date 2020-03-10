/*
 *   Copyright (c) 2020
 *   All rights reserved.
 */

#[macro_export]
macro_rules! native_fn {
    ($worker: ident,$state: ident,$proc: ident => $name: ident ($arg: ident) $e: expr ) => {
        pub extern "C" fn $name(
            $worker: &mut ProcessWorker,
            $state: &RcState,
            $proc: &Arc<Process>,
            _: Value,
            args: &[Value],
        ) -> Result<ReturnValue, Value> {
            let $arg = args[0];
            $e
        }
    };
    ($worker: ident,$state: ident,$proc: ident => $name: ident ($arg: ident,$arg2: ident) $e: expr ) => {
        pub extern "C" fn $name(
            $worker: &mut ProcessWorker,
            $state: &RcState,
            $proc: &Arc<Process>,
            _: Value,
            args: &[Value],
        ) -> Result<ReturnValue, Value> {
            let $arg = args[0];
            let $arg2 = args[1];
            $e
        }
    };
    ($worker: ident,$state: ident,$proc: ident => $name: ident (...$args: ident) $e: expr ) => {
        pub extern "C" fn $name(
            $worker: &mut ProcessWorker,
            $state: &RcState,
            $proc: &Arc<Process>,
            _: Value,
            $args: &[Value],
        ) -> Result<ReturnValue, Value> {
            $e
        }
    };
    ($worker: ident,$state: ident,$proc: ident => $name: ident $this: ident ($arg: ident,$arg2: ident) $e: expr ) => {
        pub extern "C" fn $name(
            $worker: &mut ProcessWorker,
            $state: &RcState,
            $proc: &Arc<Process>,
            this: Value,
            args: &[Value],
        ) -> Result<ReturnValue, Value> {
            let $arg = args[0];
            let $arg2 = args[1];
            let $this = this;
            $e
        }
    };
    ($worker: ident,$state: ident,$proc: ident => $name: ident $this: ident ($arg: ident) $e: expr ) => {
        pub extern "C" fn $name(
            $worker: &mut ProcessWorker,
            $state: &RcState,
            $proc: &Arc<Process>,
            this: Value,
            args: &[Value],
        ) -> Result<ReturnValue, Value> {
            let $arg = args[0];
            //let $arg2 = args[1];
            let $this = this;
            $e
        }
    };
    ($worker: ident,$state: ident,$proc: ident => $name: ident $this: ident (...$args: ident) $e: expr ) => {
        pub extern "C" fn $name(
            $worker: &mut ProcessWorker,
            $state: &RcState,
            $proc: &Arc<Process>,
            this: Value,
            $args: &[Value],
        ) -> Result<ReturnValue, Value> {
            let $this = this;
            $e
        }
    };
}

pub mod array_functions;
pub mod cell;
pub mod channel;
pub mod config;
pub mod core_functions;
pub mod env_functions;
pub mod exception;
pub mod file_functions;
pub mod function_functions;
pub mod interner;
pub mod io_functions;
pub mod math_object;
pub mod module;
pub mod module_functions;
pub mod number_functions;
pub mod object_functions;
pub mod process;
pub mod process_functions;
pub mod regex_functions;
pub mod scheduler;
pub mod state;
pub mod string_functions;
pub mod value;
use crate::heap::{onthefly, GCVariant};
use module::*;
use parking_lot::Mutex;
use state::*;

lazy_static::lazy_static!(
    pub static ref RUNTIME: Runtime = Runtime::new();
);

pub struct Runtime {
    pub state: RcState,
    pub registry: Mutex<ModuleRegistry>,
}

impl Runtime {
    pub fn new() -> Self {
        let state = State::new(config::Config::default());
        let rt = Self {
            state: state.clone(),
            registry: Mutex::new(ModuleRegistry::new(state)),
        };
        rt.initialize_builtins();
        rt
    }

    pub fn initialize_builtins(&self) {
        process_functions::initialize_process_prototype(&self.state);
        io_functions::initialize_io(&self.state);
        array_functions::initialize_array_builtins(&self.state);
        core_functions::initialize_core(&self.state);
        object_functions::initialize_object(&self.state);
        module_functions::initialize_module(&self.state);
        exception::initialize_exception(&self.state);
        regex_functions::initialize_regex(&self.state);
        env_functions::initialize_env(&self.state);
        math_object::initialize_math(&self.state);
        string_functions::initialize_string(&self.state);
        function_functions::initialize_function(&self.state);
        number_functions::initialize_number(&self.state);
        file_functions::initialize_file(&self.state);
    }

    pub fn start_pools(&self) {
        if let GCVariant::OnTheFly = self.state.config.gc {
            let lock = onthefly::GC.lock();
            lock.mark_pool.start(self.state.clone());
            lock.sweeper.start(self.state.clone());
        }
        let gguard = self.state.gc_pool.start(self.state.clone());
        self.state.scheduler.blocking_pool.start();
        let pguard = self.state.scheduler.primary_pool.start_main();
        let state = self.state.clone();
        std::thread::spawn(move || {
            state.timeout_worker.run(&state.scheduler);
        });
        pguard.join().unwrap();
        gguard.join().unwrap();
    }
}
