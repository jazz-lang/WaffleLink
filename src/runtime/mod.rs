pub mod cell;
pub mod channel;
pub mod config;
pub mod interner;
pub mod module;
pub mod process;
pub mod process_functions;
pub mod scheduler;
pub mod state;
pub mod value;

use state::*;

lazy_static::lazy_static!(
    pub static ref RUNTIME: Runtime = Runtime::new();
);

pub struct Runtime {
    pub state: RcState,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            state: State::new(config::Config::default()),
        }
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
