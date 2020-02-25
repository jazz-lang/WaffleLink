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

extern crate regalloc as ra;

pub const REAL_REGS_END: usize = 32;

use crate::bytecode::basicblock::BasicBlock;
use ra::Function as RFunction;
use ra::{
    BlockIx, InstIx, InstRegUses, Map, MyRange, RealReg, RealRegUniverse, Reg, RegClass,
    RegClassInfo, Set, SpillSlot, TypedIxVec, VirtualReg, Writable, NUM_REG_CLASSES,
};
#[derive(Debug, Copy)]
pub struct Block {
    pub label: usize,
    pub start: InstIx,
    pub len: u32,
}
impl Block {
    pub fn new(label: usize, start: InstIx, len: u32) -> Self {
        Self { label, start, len }
    }
}
impl Clone for Block {
    // This is only needed for debug printing.
    fn clone(&self) -> Self {
        Block {
            label: self.label.clone(),
            start: self.start,
            len: self.len,
        }
    }
}
use crate::bytecode;
use crate::util::ptr::*;
use bytecode::instruction::*;
use std::collections::HashMap;
pub struct RegisterAllocationPass {
    pub blocks: TypedIxVec<BlockIx, Block>,
    pub instructions: TypedIxVec<InstIx, Instruction>,
    stack_positions: Ptr<HashMap<usize, usize>>,
    stack: Ptr<usize>,
}

impl RegisterAllocationPass {
    pub fn new() -> Self {
        Self {
            blocks: TypedIxVec::new(),
            instructions: TypedIxVec::new(),
            stack: Ptr::new(0),
            stack_positions: Ptr::new(HashMap::new()),
        }
    }

    pub fn block(&mut self, idx: usize, mut insns: TypedIxVec<InstIx, Instruction>) {
        let start = self.instructions.len();
        let len = insns.len() as u32;
        self.instructions.append(&mut insns);
        let b = Block::new(idx, InstIx::new(start), len);
        self.blocks.push(b);
    }

    pub fn spill(&self, x: SpillSlot) -> usize {
        let pos = x.get_usize();
        let real_pos = pos + *self.stack.get();
        self.stack_positions.get().insert(pos, real_pos);
        *self.stack.get() += 1;

        real_pos
    }
    pub fn update_from_alloc(&mut self, result: ra::RegAllocResult<Self>) {
        self.instructions = TypedIxVec::from_vec(result.insns);
        let num_blocks = self.blocks.len();
        let mut i = 0;
        for bix in self.blocks.range() {
            let block = &mut self.blocks[bix];
            block.start = result.target_map[bix];
            block.len = if i + 1 < num_blocks {
                result.target_map[BlockIx::new(i + 1)].get()
            } else {
                self.instructions.len()
            } - block.start.get();
            i += 1;
        }
    }

    pub fn to_basic_blocks(&self) -> Vec<BasicBlock> {
        let instructions = self.instructions.iter().collect::<Vec<_>>();
        let mut basic_blocks = vec![];
        for (i, block) in self.blocks.iter().enumerate() {
            let insns = &instructions
                [block.start.get() as usize..block.start.get() as usize + block.len as usize];
            basic_blocks.push(BasicBlock {
                instructions: insns.iter().map(|x| (**x).clone()).collect(),
                index: i,
            });
        }
        basic_blocks
    }
}

impl RFunction for RegisterAllocationPass {
    type Inst = Instruction;

    fn is_ret(&self, ins: InstIx) -> bool {
        match &self.instructions[ins] {
            Instruction::Return(_) => true,
            _ => false,
        }
    }
    fn func_liveins(&self) -> Set<RealReg> {
        Set::empty()
    }

    fn func_liveouts(&self) -> Set<RealReg> {
        Set::empty()
    }

    fn insns(&self) -> &[Instruction] {
        self.instructions.elems()
    }

    fn insns_mut(&mut self) -> &mut [Instruction] {
        self.instructions.elems_mut()
    }

    fn get_insn(&self, ix: InstIx) -> &Instruction {
        &self.instructions[ix]
    }

    fn get_insn_mut(&mut self, ix: InstIx) -> &mut Instruction {
        &mut self.instructions[ix]
    }

    fn entry_block(&self) -> BlockIx {
        BlockIx::new(0)
    }
    fn blocks(&self) -> MyRange<BlockIx> {
        self.blocks.range()
    }
    /// Provide the range of instruction indices contained in each block.
    fn block_insns(&self, block: BlockIx) -> MyRange<InstIx> {
        MyRange::new(self.blocks[block].start, self.blocks[block].len as usize)
    }
    /// Get CFG successors: indexed by block, provide a list of successor blocks.
    fn block_succs(&self, block: BlockIx) -> Vec<BlockIx> {
        let last_insn = self.blocks[block].start.plus(self.blocks[block].len - 1);
        self.instructions[last_insn].get_targets()
    }

    /// Provide the defined, used, and modified registers for an instruction.
    fn get_regs(&self, insn: &Self::Inst) -> regalloc::InstRegUses {
        let (d, m, u) = insn.get_reg_usage();

        InstRegUses {
            used: u,
            defined: Set::from_vec(d.iter().map(|x| Writable::from_reg(*x)).collect::<Vec<_>>()),
            modified: Set::from_vec(m.iter().map(|x| Writable::from_reg(*x)).collect::<Vec<_>>()),
        }
    }
    fn map_regs(
        insn: &mut Self::Inst,
        pre_map: &Map<VirtualReg, RealReg>,
        post_map: &Map<VirtualReg, RealReg>,
    ) {
        insn.map_regs_d_u(
            /* define-map = */ post_map, /* use-map = */ pre_map,
        );
    }

    fn is_move(&self, insn: &Instruction) -> Option<(Writable<Reg>, Reg)> {
        match insn {
            Instruction::Move(dst, src) => Some((
                Writable::from_reg(if *dst > 32 {
                    Reg::new_virtual(RegClass::I64, *dst as _)
                } else {
                    Reg::new_real(RegClass::I64, 1, *dst as _)
                }),
                Reg::new_virtual(RegClass::I64, *src as _),
            )),
            _ => None,
        }
    }

    fn get_spillslot_size(&self, _: RegClass, _: VirtualReg) -> u32 {
        // For our VM, every value occupies one spill slot.
        1
    }

    fn gen_spill(&self, to_slot: SpillSlot, from_reg: RealReg, _: VirtualReg) -> Instruction {
        let real_addr = self.spill(to_slot);
        Instruction::StoreStack(from_reg.to_reg().get_index() as _, real_addr as _)
    }

    fn gen_reload(&self, to: Writable<RealReg>, slot: SpillSlot, _: VirtualReg) -> Instruction {
        Instruction::LoadStack(to.to_reg().get_index() as _, slot.get_usize() as u32)
    }

    fn gen_move(&self, to: Writable<RealReg>, from: RealReg, _: VirtualReg) -> Instruction {
        Instruction::Move(to.to_reg().get_index() as _, from.get_index() as _)
    }

    fn maybe_direct_reload(
        &self,
        insn: &Self::Inst,
        _: VirtualReg,
        _: SpillSlot,
    ) -> Option<Self::Inst> {
        None
    }
}

fn make_universe() -> RealRegUniverse {
    const REG_COUNT: usize = 32;
    let mut regs = Vec::<(RealReg, String)>::new();
    let mut allocable_by_class = [None; NUM_REG_CLASSES];
    let mut index = 0u8;
    let first = index as usize;
    for i in 0..REG_COUNT {
        let name = format!("%r{}", i).to_string();
        let reg = Reg::new_real(RegClass::I64, /*enc=*/ 0, index).to_real_reg();
        regs.push((reg, name));
        index += 1;
        let last = index as usize - 1;
        allocable_by_class[RegClass::I64.rc_to_usize()] = Some(RegClassInfo {
            first,
            last,
            suggested_scratch: Some(last),
        });
    }

    let allocable = regs.len();
    let univ = RealRegUniverse {
        regs,
        // all regs are allocable
        allocable,
        allocable_by_class,
    };
    univ.check_is_sane();

    univ
}
use crate::runtime::cell::*;
use crate::util::arc::Arc;
impl super::BytecodePass for RegisterAllocationPass {
    fn execute(&mut self, f: &mut Arc<Vec<BasicBlock>>) {
        for (i, block) in f.iter().enumerate() {
            self.block(i, TypedIxVec::from_vec(block.instructions.clone()));
        }
        let algo = ra::RegAllocAlgorithm::Backtracking;
        let result = ra::allocate_registers(self, algo, &make_universe()).unwrap();
        f.clear();
        self.update_from_alloc(result);
        **f = self.to_basic_blocks();
    }
}
