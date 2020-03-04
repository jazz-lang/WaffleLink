pub struct TailCallEliminationPass;

use super::*;
use crate::bytecode;
use crate::util::arc::Arc;
use std::collections::{BTreeSet, HashSet};

impl BytecodePass for TailCallEliminationPass {
    fn execute(&mut self,f: &mut Arc<Vec<BasicBlock>>) {
        for bb in f.iter_mut() {
            // The last instruction is always a Return or Branch to Return instruction, so we check the
            // instruction that preceeds it.
            if bb.instructions.is_empty() || bb.instructions.len() < 2 {
                continue;
            }
            let pos = bb.instructions.len() - 2;

            let ins = bb.instructions.get(pos);
            if ins.is_none() {
                continue;
            } else {
                let ins = ins.copied().unwrap();
                if let Instruction::Call(dest,callee,argc) = ins {
                    bb.instructions[pos] = Instruction::TailCall(dest,callee,argc);
                }
            }
        }
    }
}
