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

use super::value::*;
use crate::util::arc::Arc;
use std::vec::Vec;
pub struct Module {
    pub name: Arc<String>,
    pub globals: Vec<Value>,
    pub main_fn: Value,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Self {
            name: Arc::new(name.to_owned()),
            globals: vec![],
            main_fn: Value::empty(),
        }
    }
    pub fn get_global_at(&self, id: usize) -> Value {
        self.globals
            .get(id)
            .map(|x| *x)
            .unwrap_or(Value::from(VTag::Undefined))
    }
}
