#![feature(optimize_attribute)]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(naked_functions)]
#![allow(unused_assignments)]

pub mod arc;
pub mod gc;
pub mod interp;
pub mod lock;
pub mod module;
pub mod opcodes;
pub mod pure_nan;
pub mod state;
pub mod threads;
pub mod transition;
pub mod value;
use arc::ArcWithoutWeak as Arc;
use parking_lot::Mutex;
use std::io::Read;
pub struct Globals {
    map: dashmap::DashMap<String, value::Value>,
}
impl Globals {
    pub fn new() -> Self {
        Self {
            map: dashmap::DashMap::new(),
        }
    }
    pub fn get(&self, key: &str) -> Option<value::Value> {
        self.map.get(key).map(|item| *item.value())
    }
    pub fn set(&self, key: &str, value: value::Value) -> bool {
        if let Some(mut prev) = self.map.get_mut(key) {
            *prev.value_mut() = value;
            false
        } else {
            self.map.insert(key.to_owned(), value);
            true
        }
    }
}
use gc::*;
use threads::Threads;
pub struct Machine {
    pub threads: Threads,
    pub globals: Globals,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            threads: Threads::new(),
            globals: Globals::new(),
        }
    }

    pub fn allocate<T: Collectable + 'static>(&self, val: T) -> Root<T> {
        HEAP.allocate(self, val)
    }
}

lazy_static::lazy_static! {
    pub static ref HEAP: Arc<WaffleHeap> = Arc::new(WaffleHeap::new());
    pub static ref MACHINE: Arc<Machine> = Arc::new(Machine::new());
    pub static ref MODULE_REGISTRY: Arc<ModuleRegistry> = Arc::new(ModuleRegistry::new());
}
use std::collections::HashMap;
pub struct ModuleRegistry {
    parsed: Mutex<HashMap<String, value::Value>>,
}
impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            parsed: Mutex::new(Default::default()),
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.parsed.lock().contains_key(name)
    }

    pub fn parsed(&self) -> Vec<value::Value> {
        self.parsed.lock().iter().map(|x| *x.1).collect()
    }
    pub fn get(&self, name: &str) -> Option<value::Value> {
        self.parsed.lock().get(name).map(|r| *r)
    }

    fn find_path(&self, path: &str) -> Result<String, String> {
        let mut input_path = std::path::PathBuf::from(path);
        if input_path.is_relative() {
            let mut found = false;

            /*for directory in self.state.config.directories.iter() {
                let full_path = directory.join(path);

                if full_path.exists() {
                    input_path = full_path;
                    found = true;

                    break;
                }
            }*/

            if !found {
                return Err(format!("Module '{}' doesn't exist", path.to_string()));
            }
        }
        Ok(input_path.to_str().unwrap().to_owned())
    }

    pub fn parse_module(&mut self, name: &str, path: &str) -> Result<value::Value, String> {
        match std::fs::File::open(path) {
            Ok(mut file) => {
                let mut contents = vec![];
                file.read_to_end(&mut contents).map_err(|e| e.to_string())?;
                /*let mut reader = BytecodeReader {
                    bytes: std::io::Cursor::new(&contents),
                };
                let mut module = reader.read_module();
                module.name = Value::from(self.state.intern_string(name.to_owned()));
                let module_cell = self.state.allocate(Cell::with_prototype(
                    CellValue::Module(module),
                    self.state.module_prototype.as_cell(),
                ));
                Ok(Value::from(module_cell))*/
                unimplemented!()
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn load(&mut self, name: &str, path: &str) -> Result<(value::Value, bool), String> {
        let p = self.parsed.lock();
        if !p.contains_key(name) {
            let full_path = self.find_path(path)?;
            drop(p);
            self.parse_module(name, &full_path)
                .map(|module| (module, true))
        } else {
            Ok((p.get(name).map(|x| *x).unwrap(), false))
        }
    }
}
