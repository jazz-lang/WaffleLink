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

pub mod graph_coloring;
pub mod liveness;

pub struct RegisterAllocation;

use crate::bytecode::*;
use crate::runtime::cell::Function;
use crate::util::arc::Arc;
use basicblock::*;
use cfg::*;
use instruction::*;
use passes::BytecodePass;

impl RegisterAllocation {
    pub fn new() -> Self {
        Self
    }
    fn coloring(&mut self, cf: &mut Arc<Function>) {
        let cfg = build_cfg_for_code(&cf.code);
        let a = loopanalysis::loopanalysis(cf, &cfg);
        let coloring = graph_coloring::GraphColoring::start(&mut cf.code, &a);
        log::trace!("Replacing Registers...");
        for (temp, machine_reg) in coloring.get_assignments() {
            log::trace!("replacing {} with {}", temp, machine_reg);
            for bb in coloring.cf.iter_mut() {
                for i in bb.instructions.iter_mut() {
                    i.replace_reg(temp, machine_reg);
                }
            }
            //coloring.cf.mc_mut().replace_reg(temp, machine_reg);
            //coloring.cf.temps.insert(temp, machine_reg);
        }
    }
}

impl BytecodePass for RegisterAllocation {
    fn execute(&mut self, f: &mut Arc<Function>) {
        self.coloring(f);
    }
}
