use super::*;
use crate::runtime::cell::Function;
use crate::util::arc::Arc;
use std::collections::HashMap;
// TODO: This pass does not work as it should.
pub struct CSEPass;

impl BytecodePass for CSEPass {
    fn execute(&mut self, f: &mut Arc<Vec<BasicBlock>>) {
        let mut ins_map = HashMap::new();
        let mut stats = 0;
        for bb in f.iter_mut() {
            for i in bb.instructions.iter_mut() {
                let k = if let Instruction::Binary { .. } = i {
                    *i
                } else {
                    match i {
                        Instruction::LoadInt { .. }
                        | Instruction::LoadNumber { .. }
                        | Instruction::LoadTrue { .. }
                        | Instruction::LoadFalse { .. }
                        | Instruction::LoadNull { .. }
                        | Instruction::LoadUndefined { .. } => *i,
                        _ => continue,
                    }
                };
                if ins_map.contains_key(&k) {
                    let ins_new = ins_map[&k];
                    *i = ins_new; // this is wrong, we should replace all `i` uses.
                    stats += 1;
                } else {
                    ins_map.insert(k, *i);
                }
            }
        }
        if stats > 0 {
            log::debug!("Replaced {} instructions", stats);
        }
    }
}
