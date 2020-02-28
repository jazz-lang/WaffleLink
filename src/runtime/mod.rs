/*
 *   Copyright (c) 2020
 *   All rights reserved.
 */

pub mod array_functions;
pub mod cell;
pub mod channel;
pub mod config;
pub mod interner;
pub mod io_functions;
pub mod module;
pub mod process;
pub mod process_functions;
pub mod scheduler;
pub mod state;
pub mod value;
pub mod core_functions;
use module::*;
use state::*;
use parking_lot::Mutex;

lazy_static::lazy_static!(
    pub static ref RUNTIME: Runtime = Runtime::new();
);

pub struct Runtime {
    pub state: RcState,
    pub registry: Mutex<ModuleRegistry>
}

impl Runtime {
    pub fn new() -> Self {
        let state = State::new(config::Config::default());
        let rt = Self {
            state: state.clone(),
            registry: Mutex::new(ModuleRegistry::new(state))
        };
        rt.initialize_builtins();
        rt
    }

    pub fn initialize_builtins(&self) {
        process_functions::initialize_process_prototype(&self.state);
        io_functions::initialize_io(&self.state);
        array_functions::initialize_array_builtins(&self.state);
    }

    pub fn start_pools(&self) {
        self.state.scheduler.blocking_pool.start();
        let pguard = self.state.scheduler.primary_pool.start_main();
        let state = self.state.clone();
        std::thread::spawn(move || {
            state.timeout_worker.run(&state.scheduler);
        });
        pguard.join().unwrap();
    }
}
