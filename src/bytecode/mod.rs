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

pub mod arithprofile;
pub mod basicblock;
pub mod instruction;
pub mod passes;
pub mod reader;
pub mod regalloc;
pub mod writer;

use passes::BytecodePass;

use crate::runtime::{cell::*, module::*};
use crate::util::arc::Arc;

/// Bytecode optimization level
///
/// - Slow: Not recommended for use since it does not do tail call elimination and does not simplify CFG resulting in big bytecode size
/// and probably stack overflows when recursion occurs.
/// - Fast: Recommended, this opt level does not waste a lot of time to do optimizations and does CFG simplification and tail call elimination.
/// - Slow: Not recommended, this opt level is unstable and may cause bugs in programs. Does inlining,CSE,rematerialization and some form of alias analysing.
pub enum OptLevel {
    /// Just do register allocation
    Slow,
    /// This pass includes CFG simplifier, tail call elimination and RetSink pass.
    Fast,
    /// This pass is not recommended since it's **really** unstable and may cause program crashes.
    /// Includes function inlining,CSE, rematerialization and some form of alias analysing.
    VeryFast,
}

pub fn prelink_module(m: &Arc<Module>, opt_level: OptLevel) {
    for f in m.globals.iter() {
        if f.is_cell() {
            if let CellValue::Function(ref mut f) = f.as_cell().get_mut().value {
                match opt_level {
                    OptLevel::Slow => {
                        let mut ra = regalloc::RegisterAllocation::new();
                        ra.execute(f);
                    }
                    OptLevel::Fast => {
                        let mut simplify = passes::simplify::SimplifyCFGPass;
                        //simplify.execute(f);
                        //let mut cfold = passes::constant_folding::ConstantFoldingPass;
                        //cfold.execute(f);
                        let mut ra = regalloc::RegisterAllocation::new();
                        ra.execute(f);
                        simplify.execute(f);

                        let mut peephole = passes::peephole::PeepholePass;
                        peephole.execute(f);
                        let mut tcall = passes::tail_call_elim::TailCallEliminationPass;
                        let mut ret_sink = passes::ret_sink::RetSink;
                        ret_sink.execute(f);
                        simplify.execute(f);
                        peephole.execute(f);
                        tcall.execute(f);
                    }
                    OptLevel::VeryFast => {
                        // TODO: Invoke CSE, alias analysing.
                        let mut simplify = passes::simplify::SimplifyCFGPass;
                        simplify.execute(f);
                        simplify.execute(f);
                        passes::simple_inlining::do_inlining(f, m);
                        let mut ra = regalloc::RegisterAllocation::new();
                        ra.execute(f);

                        let mut tcall = passes::tail_call_elim::TailCallEliminationPass;
                        tcall.execute(f);
                        let mut simplify = passes::simplify::SimplifyCFGPass;
                        simplify.execute(f);
                        let mut ret_sink = passes::ret_sink::RetSink;
                        ret_sink.execute(f);
                        let mut peephole = passes::peephole::PeepholePass;
                        peephole.execute(f);
                    }
                }
            }
        }
    }
}
