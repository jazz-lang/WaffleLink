use crate::bytecode;
use bytecode::def::*;
use bytecode::virtual_reg::*;
use regalloc::{
    BlockIx, Function, InstIx, Range, Reg, RegClass, RegUsageCollector, RegUsageMapper, TypedIxVec,
    Writable,
};
macro_rules! r {
    ($x: expr) => {
        if $x.to_local() <= 128 {
            assert!($x.to_local() != 255);
            Reg::new_real(RegClass::I32, 0, $x.to_local() as _)
        } else {
            Reg::new_virtual(RegClass::I32, $x.to_local() as _)
        }
    };
}
fn get_usage(ins: Ins, collector: &mut RegUsageCollector) {
    if let Ins::Mov { dst, src } = ins {
        if dst.is_local() {
            collector.add_mod(Writable::from_reg(r!(dst)));
        }
        if src.is_local() {
            collector.add_use(r!(dst));
        }
        return;
    }
    let uses = ins.get_uses();
    for u in uses {
        if u.is_local() {
            collector.add_def(Writable::from_reg(r!(u)));
        }
    }

    let defs = ins.get_defs();
    for def in defs {
        if def.is_local() {
            collector.add_use(r!(def))
        }
    }
}

fn get_targets(ins: Ins) -> Vec<BlockIx> {
    match ins {
        Ins::Jump { dst } => vec![BlockIx::new(dst)],
        Ins::JumpConditional {
            if_false, if_true, ..
        } => vec![BlockIx::new(if_true), BlockIx::new(if_false)],
        Ins::TryCatch { try_, catch, .. } => vec![BlockIx::new(try_), BlockIx::new(catch)],
        Ins::Return { .. } => vec![],
        _ => panic!(),
    }
}
macro_rules! v {
    ($x: expr) => {
        Reg::new_virtual(RegClass::I32, $x as u32)
            .as_virtual_reg()
            .unwrap()
    };
}

impl VirtualRegister {
    fn is_virt(&self) -> bool {
        self.is_local() && self.to_local() >= 256
    }
    fn apply_mods(&mut self, mapper: &RegUsageMapper) {
        self.apply(|vreg| mapper.get_mod(vreg));
    }
    fn apply_defs(&mut self, mapper: &RegUsageMapper) {
        self.apply(|vreg| mapper.get_def(vreg));
    }
    fn apply_uses(&mut self, mapper: &RegUsageMapper) {
        self.apply(|vreg| mapper.get_use(vreg));
    }
    fn apply<F: Fn(regalloc::VirtualReg) -> Option<regalloc::RealReg>>(&mut self, f: F) {
        if self.is_virt() {
            if let Some(rreg) = f(v!(self.to_local())) {
                *self = VirtualRegister::tmp(rreg.get_index() as _);
            }
        }
    }
}

fn map_usage(ins: &mut Ins, mapper: &RegUsageMapper) {
    use Ins::*;
    match ins {
        Ins::Mov { dst, src, .. } => {
            dst.apply_mods(mapper);
            src.apply_uses(mapper);
        }
        Ins::LoadI32 { dst, .. } => {
            dst.apply_defs(mapper);
        }
        Ins::NewGeneratorFunction { dst, src } => {
            dst.apply_defs(mapper);
            src.apply_uses(mapper);
        }
        Ins::CloseEnv { dst, function, .. } => {
            dst.apply_defs(mapper);
            function.apply_uses(mapper);
        }
        Ins::Call {
            dst,
            function,
            this,
            ..
        } => {
            dst.apply_defs(mapper);
            function.apply_uses(mapper);
            this.apply_uses(mapper);
        }
        Ins::TailCall {
            dst,
            function,
            this,
            ..
        } => {
            dst.apply_defs(mapper);
            function.apply_uses(mapper);
            this.apply_uses(mapper);
        }
        Ins::Yield { dst, res } => {
            dst.apply_defs(mapper);
            res.apply_uses(mapper);
        }
        Ins::TryCatch { reg, .. } => {
            reg.apply_defs(mapper);
        }
        Ins::Await { dst, on } => {
            dst.apply_defs(mapper);
            on.apply_uses(mapper);
        }
        Add { lhs, src, dst, .. }
        | Sub { lhs, src, dst, .. }
        | Div { lhs, src, dst, .. }
        | Mul { lhs, src, dst, .. }
        | Mod { lhs, src, dst, .. }
        | Concat { lhs, src, dst, .. }
        | Shr { lhs, src, dst, .. }
        | Shl { lhs, src, dst, .. }
        | UShr { lhs, src, dst, .. }
        | Eq { lhs, src, dst, .. }
        | NEq { lhs, src, dst, .. }
        | Greater { lhs, src, dst, .. }
        | GreaterEq { lhs, src, dst, .. }
        | Less { lhs, src, dst, .. }
        | LessEq { lhs, src, dst, .. } => {
            dst.apply_defs(mapper);
            src.apply_uses(mapper);
            lhs.apply_uses(mapper);
        }
        LoadGlobal { dst, name, .. } => {
            dst.apply_defs(mapper);
            name.apply_uses(mapper);
        }
        JumpConditional { cond, .. } => {
            cond.apply_uses(mapper);
        }
        Ins::IteratorOpen { dst, iterable } => {
            dst.apply_defs(mapper);
            iterable.apply_uses(mapper);
        }
        Ins::IteratorNext {
            next,
            done,
            value,
            iterator,
        } => {
            next.apply_defs(mapper);
            done.apply_defs(mapper);
            value.apply_defs(mapper);
            iterator.apply_uses(mapper);
        }
        Ins::LoadUp { dst, .. } => {
            dst.apply_defs(mapper);
        }
        Ins::GetById { dst, base, id, .. } => {
            dst.apply_defs(mapper);
            base.apply_uses(mapper);
            id.apply_uses(mapper);
        }
        Ins::PutById { val, base, id, .. } => {
            val.apply_uses(mapper);
            base.apply_uses(mapper);
            id.apply_uses(mapper);
        }
        Ins::GetByVal { dst, base, val } => {
            dst.apply_defs(mapper);
            base.apply_uses(mapper);
            val.apply_uses(mapper);
        }
        Ins::PutByVal { src, base, val } => {
            src.apply_uses(mapper);
            base.apply_uses(mapper);
            val.apply_uses(mapper);
        }
        Ins::LoadThis { dst } => {
            dst.apply_defs(mapper);
        }
        Ins::NewObject { dst } => {
            dst.apply_defs(mapper);
        }
        Construct { dst, obj, .. } | ConstructNoArgs { dst, obj, .. } => {
            dst.apply_defs(mapper);
            obj.apply_uses(mapper);
        }
        _ => (),
    }
}

#[derive(Copy, Clone)]
pub struct Block {
    start: InstIx,
    len: u32,
}

pub struct Func {
    blocks: TypedIxVec<BlockIx, Block>,
    insns: TypedIxVec<InstIx, Ins>,
    vreg_count: usize,
}
impl Function for Func {
    type Inst = Ins;
    fn blocks(&self) -> Range<BlockIx> {
        self.blocks.range()
    }
    fn insns(&self) -> &[Self::Inst] {
        self.insns.elems()
    }
    fn insns_mut(&mut self) -> &mut [Self::Inst] {
        self.insns.elems_mut()
    }
    fn get_insn(&self, insn: InstIx) -> &Self::Inst {
        &self.insns[insn]
    }

    fn get_insn_mut(&mut self, insn: InstIx) -> &mut Self::Inst {
        &mut self.insns[insn]
    }
    fn entry_block(&self) -> BlockIx {
        BlockIx::new(0)
    }
    fn block_insns(&self, block: BlockIx) -> Range<InstIx> {
        Range::new(self.blocks[block].start, self.blocks[block].len as usize)
    }
    fn block_succs(&self, block: BlockIx) -> std::borrow::Cow<[BlockIx]> {
        let last_insn = self.blocks[block].start.plus(self.blocks[block].len - 1);
        std::borrow::Cow::Owned(get_targets(self.insns[last_insn]))
    }

    fn is_ret(&self, insn: InstIx) -> bool {
        match &self.insns[insn] {
            Ins::Return { .. } => true,
            Ins::Throw { .. } => true,
            _ => false,
        }
    }

    fn get_regs(insn: &Self::Inst, collector: &mut RegUsageCollector) {
        get_usage(*insn, collector);
    }

    fn map_regs(insn: &mut Self::Inst, maps: &RegUsageMapper) {
        map_usage(insn, maps);
    }
    fn is_move(&self, insn: &Self::Inst) -> Option<(Writable<Reg>, Reg)> {
        match insn {
            &Ins::Mov { dst, src } => {
                if dst.is_local() && src.is_local() {
                    Some((Writable::from_reg(r!(dst)), r!(src)))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    fn get_vreg_count_estimate(&self) -> Option<usize> {
        None
    }
    fn get_spillslot_size(&self, _regclass: RegClass, _for_vreg: regalloc::VirtualReg) -> u32 {
        0
    }

    fn gen_spill(
        &self,
        to_slot: regalloc::SpillSlot,
        from_reg: regalloc::RealReg,
        for_vreg: regalloc::VirtualReg,
    ) -> Self::Inst {
        panic!("spills not supported");
    }
    fn gen_reload(
        &self,
        _to_reg: Writable<regalloc::RealReg>,
        _from_slot: regalloc::SpillSlot,
        _for_vreg: regalloc::VirtualReg,
    ) -> Self::Inst {
        panic!("spills not supported");
    }

    fn gen_move(
        &self,
        to_reg: Writable<regalloc::RealReg>,
        from_reg: regalloc::RealReg,
        _for_vreg: regalloc::VirtualReg,
    ) -> Self::Inst {
        Ins::Mov {
            dst: VirtualRegister::tmp(to_reg.to_reg().to_reg().get_index() as _),
            src: VirtualRegister::tmp(from_reg.get_index() as _),
        }
    }
    fn gen_zero_len_nop(&self) -> Self::Inst {
        Ins::Nop
    }
    fn maybe_direct_reload(
        &self,
        _insn: &Self::Inst,
        _reg: regalloc::VirtualReg,
        _slot: regalloc::SpillSlot,
    ) -> Option<Self::Inst> {
        None
    }
    fn func_liveins(&self) -> regalloc::Set<regalloc::RealReg> {
        regalloc::Set::empty()
    }
    fn func_liveouts(&self) -> regalloc::Set<regalloc::RealReg> {
        regalloc::Set::empty()
    }
}
fn make_universe() -> regalloc::RealRegUniverse {
    let total_regs = 255;
    let mut regs = Vec::<(regalloc::RealReg, String)>::new();
    let mut allocable_by_class = [None; regalloc::NUM_REG_CLASSES];
    let mut index = 0u8;
    let first = index as usize;
    for i in 0..128 {
        let name = format!("loc{}", i).to_string();
        let reg = Reg::new_real(RegClass::I32, /*enc=*/ 0, index).to_real_reg();
        regs.push((reg, name));
        assert!(index < 200);
        index += 1;
    }
    let last = index as usize - 1;
    allocable_by_class[RegClass::I32.rc_to_usize()] = Some(regalloc::RegClassInfo {
        first,
        last,
        suggested_scratch: None,
    });
    let univ = regalloc::RealRegUniverse {
        allocable: regs.len(),
        regs,
        allocable_by_class,
    };
    univ.check_is_sane();
    univ
}

use bytecode::*;

fn make_func(from: &CodeBlock) -> Func {
    let mut blocks = TypedIxVec::new();
    let mut insns = TypedIxVec::new();
    let mut block = |idx, mut instructions: TypedIxVec<InstIx, Ins>| {
        let start = insns.len();
        let len = instructions.len() as u32;
        insns.append(&mut instructions);
        let b = Block {
            start: InstIx::new(start as _),
            len,
        };
        blocks.push(b);
    };

    for bb in from.code.iter() {
        block(bb.id as usize, TypedIxVec::from_vec(bb.code.clone()));
    }
    Func {
        insns,
        blocks,
        vreg_count: 0,
    }
}
impl Func {
    fn update(&mut self, result: regalloc::RegAllocResult<Self>) {
        self.insns = TypedIxVec::from_vec(result.insns);
        let num_blocks = self.blocks.len();
        let mut i = 0;
        for bix in self.blocks.range() {
            let block = &mut self.blocks[bix];
            block.start = result.target_map[bix];
            block.len = if i + 1 < num_blocks {
                result.target_map[BlockIx::new(i + 1)].get()
            } else {
                self.insns.len()
            } - block.start.get();
            i += 1;
        }
    }
    pub fn to_basic_blocks(&self) -> Vec<BasicBlock> {
        let instructions = self.insns.iter().collect::<Vec<_>>();
        let mut basic_blocks = vec![];
        for (i, block) in self.blocks.iter().enumerate() {
            let insns = &instructions
                [block.start.get() as usize..block.start.get() as usize + block.len as usize];
            basic_blocks.push(BasicBlock {
                code: insns.iter().map(|x| (**x).clone()).collect(),
                id: i as _,
                livein: vec![],
                liveout: vec![],
            });
        }
        basic_blocks
    }
}
pub fn run_minira(
    on: &mut CodeBlock,
    algo: regalloc::AlgorithmWithDefaults,
    rt: &mut crate::runtime::Runtime,
) {
    let univ = make_universe();
    if log::log_enabled!(log::Level::Trace) {
        log::trace!("Before regalloc:");
        let mut buf = String::new();
        on.dump(&mut buf, rt);
        println!("{}", buf);
    }
    let mut f = make_func(on);
    let result = regalloc::allocate_registers(&mut f, &univ, algo);
    match result {
        Ok(r) => f.update(r),
        Err(e) => {
            panic!("{}", e.to_string());
        }
    }

    on.code = f.to_basic_blocks();
}
