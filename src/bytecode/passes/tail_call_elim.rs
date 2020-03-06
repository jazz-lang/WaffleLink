/// This pass replaces all calls in tail position to tail calls.
/// Since Waffle interpreter does not support mutual recursion this step is necessary if we want
/// to handle recursion and get good execution speed.
pub struct TailCallEliminationPass;

use super::*;
use crate::bytecode;
use crate::util::arc::Arc;

impl BytecodePass for TailCallEliminationPass {
    fn execute(&mut self, f: &mut Arc<Function>) {
        let f = &mut f.code;
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
                if let Instruction::Call(dest, callee, argc) = ins {
                    bb.instructions[pos] = Instruction::TailCall(dest, callee, argc);
                }
            }
        }
    }
}
