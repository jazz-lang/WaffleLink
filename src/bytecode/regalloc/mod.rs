pub mod graph_coloring;
pub mod liveness;

pub struct RegisterAllocation;

use crate::bytecode::*;
use crate::runtime::cell::Function;
use crate::util::arc::Arc;
use basicblock::*;
use instruction::*;
use passes::BytecodePass;

impl RegisterAllocation {
    pub fn new() -> Self {
        Self
    }
    fn coloring(&mut self, cf: &mut Arc<Vec<BasicBlock>>) {
        let coloring = graph_coloring::GraphColoring::start(cf);
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
        self.coloring(&mut f.code);
    }
}
