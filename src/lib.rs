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

#![cfg_attr(all(feature = "nightly", feature = "threaded"), feature(asm))]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
extern crate intrusive_collections;
#[macro_use]
pub mod util;
pub mod bytecode;
pub mod heap;
pub mod interpreter;
pub mod jit;
pub mod runtime;
pub mod types;
pub use runtime::cell::ReturnValue;
