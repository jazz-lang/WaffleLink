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
use super::state::*;
use super::value::*;
use crate::bytecode::reader::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::vec::Vec;
pub struct Module {
    pub name: Value,
    pub globals: Vec<Value>,
    pub main_fn: Value,
    pub exports: Value,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Self {
            name: Value::from(super::RUNTIME.state.intern_string(name.to_owned())),
            globals: vec![],
            main_fn: Value::empty(),
            exports: Value::empty(),
        }
    }
    pub fn get_global_at(&self, id: usize) -> Value {
        self.globals
            .get(id)
            .map(|x| *x)
            .unwrap_or(Value::from(VTag::Undefined))
    }
}

pub struct ModuleRegistry {
    state: RcState,
    parsed: HashMap<String, Value>,
}
impl ModuleRegistry {
    pub fn new(state: RcState) -> Self {
        Self {
            state,
            parsed: HashMap::new(),
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.parsed.contains_key(name)
    }

    pub fn parsed(&self) -> Vec<Value> {
        self.parsed.iter().map(|(_, value)| *value).collect()
    }
    pub fn get(&self, name: &str) -> Option<Value> {
        self.parsed.get(name).copied()
    }

    fn find_path(&self, path: &str) -> Result<String, String> {
        let mut input_path = PathBuf::from(path);
        if input_path.is_relative() {
            let mut found = false;

            for directory in self.state.config.directories.iter() {
                let full_path = directory.join(path);

                if full_path.exists() {
                    input_path = full_path;
                    found = true;

                    break;
                }
            }

            if !found {
                return Err(format!("Module '{}' doesn't exist", path.to_string()));
            }
        }
        Ok(input_path.to_str().unwrap().to_owned())
    }

    pub fn parse_module(&mut self, _name: &str, path: &str) -> Result<Value, String> {
        match File::open(path) {
            Ok(mut file) => {
                let mut contents = vec![];
                file.read_to_end(&mut contents).map_err(|e| e.to_string())?;
                let mut reader = BytecodeReader {
                    bytes: std::io::Cursor::new(&contents),
                };
                let p = std::path::Path::new(path);
                let name = p.file_name().unwrap().to_str().unwrap();
                let mut module = reader.read_module();

                module.name = Value::from(self.state.intern_string(name.to_owned()));
                let module_cell = self.state.allocate(Cell::with_prototype(
                    CellValue::Module(module),
                    self.state.module_prototype.as_cell(),
                ));
                Ok(Value::from(module_cell))
            }
            Err(e) => Err(format!("Cannot load module at '{}': {}",path,e.to_string())),
        }
    }

    pub fn load(&mut self, _name: &str, path: &str) -> Result<(Value, bool), String> {
        if !self.parsed.contains_key(path) {
            let full_path = self.find_path(path)?;
            self.parse_module(_name, &full_path)
                .map(|module| (module, true))
        } else {
            Ok((self.parsed[path], false))
        }
    }
}
