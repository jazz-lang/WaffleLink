pub mod def;
pub mod virtual_reg;
// Register numbers used in bytecode operations have different meaning according to their ranges:
//      0x80000000-0xFFFFFFFF  Negative indices from the CallFrame pointer are entries in the call frame.
//      0x00000000-0x3FFFFFFF  Forwards indices from the CallFrame pointer are local vars and temporaries with the function's callframe.
//      0x40000000-0x7FFFFFFF  Positive indices from 0x40000000 specify entries in the constant pool on the CodeBlock.
pub const FIRST_CONSTNAT_REG_INDEX: i32 = 0x40000000;

use hashlink::{LinkedHashMap, LinkedHashSet};
use indexmap::IndexMap;
pub struct BasicBlock {
    pub id: u32,
    pub code: Vec<def::Ins>,
    pub livein: Vec<virtual_reg::VirtualRegister>,
    pub liveout: Vec<virtual_reg::VirtualRegister>,
}

impl BasicBlock {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            code: vec![],
            livein: vec![],
            liveout: vec![],
        }
    }

    pub fn size(&self) -> usize {
        self.code.len()
    }

    pub fn last(&self) -> def::Ins {
        *self.code.last().unwrap()
    }
    pub fn first(&self) -> def::Ins {
        *self.code.first().unwrap()
    }
    pub fn join(&mut self, other: BasicBlock) {
        self.code.pop();
        for ins in other.code {
            self.code.push(ins);
        }
    }

    pub fn try_replace_branch_targets(&mut self, from: u32, to: u32) -> bool {
        assert!(!self.code.is_empty());
        use def::*;
        let last_ins_id = self.code.len() - 1;
        let last_ins = &mut self.code[last_ins_id];
        match *last_ins {
            Ins::JumpConditional {
                cond,
                if_true,
                if_false,
            } => {
                if if_true == from || if_false == from {
                    let if_true = if if_true == from { to } else { if_true };
                    let if_false = if if_false == from { to } else { if_false };
                    *last_ins = Ins::JumpConditional {
                        cond,
                        if_true,
                        if_false,
                    };
                    true
                } else {
                    false
                }
            }
            Ins::Jump { dst } => {
                if dst == from {
                    *last_ins = Ins::Jump { dst: to };
                    true
                } else {
                    false
                }
            }
            Ins::TryCatch { reg, try_, catch } => {
                if try_ == from || catch == from {
                    let catch = if catch == from { to } else { catch };
                    let try_ = if try_ == from { to } else { try_ };
                    *last_ins = Ins::TryCatch { reg, try_, catch };
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn branch_targets(&self) -> Vec<u32> {
        assert!(self.code.len() >= 1);
        self.code.last().unwrap().branch_targets()
    }
}

use cgc::api::{Finalizer, Traceable, Tracer};

impl Finalizer for BasicBlock {
    fn finalize(&mut self) {}
}

impl Traceable for BasicBlock {
    fn trace_with(&self, _tracer: &mut Tracer) {}
}
use crate::jit::*;
use crate::runtime::value::*;
use crate::runtime::*;
pub struct CodeBlock {
    pub constants_: Vec<Value>,
    pub constants: IndexMap<Value, usize>,
    pub arg_regs_count: u32,
    pub tmp_regs_count: u32,
    pub code: Vec<BasicBlock>,
    pub hotness: usize,
    pub cfg: Option<Box<CodeCFG>>,
    pub loopanalysis: Option<crate::bytecompiler::loopanalysis::BCLoopAnalysisResult>,
    pub jit_stub:
        Option<extern "C" fn(&mut osr::OSREntry, &mut Runtime, Value, &[Value]) -> JITResult>,
}

impl CodeBlock {
    pub fn trace_code(&self, x: bool) {
        if x {
            for bb in self.code.iter() {
                log::trace!("%{}: ", bb.id);
                for (i, ins) in bb.code.iter().enumerate() {
                    log::trace!("  [{:04}] {}", i, ins);
                }
            }
        }
    }
    pub fn dump<W: std::fmt::Write>(&self, b: &mut W, rt: &mut Runtime) -> std::fmt::Result {
        for bb in self.code.iter() {
            writeln!(b, "%{}: ", bb.id)?;
            for (i, ins) in bb.code.iter().enumerate() {
                writeln!(b, "  [{:04}] {}", i, ins)?;
            }
        }
        writeln!(b, "Constant table: ")?;
        for c in self.constants_.iter().enumerate() {
            let i = c.0;
            let c = c.1;
            writeln!(
                b,
                "id{} = {}",
                i,
                match c.to_string(rt) {
                    Ok(x) => x,
                    Err(_) => unreachable!(),
                }
            )?;
        }
        Ok(())
    }

    pub fn new_constant(&mut self, val: Value) -> virtual_reg::VirtualRegister {
        if let Some(x) = self.constants.get(&val) {
            return virtual_reg::VirtualRegister::constant(*x as i32);
        } else {
            let vreg = self.constants_.len();
            self.constants_.push(val);
            self.constants.insert(val, vreg);
            virtual_reg::VirtualRegister::constant(vreg as _)
        }
    }

    pub fn get_constant(&self, x: i32) -> Value {
        self.constants_[x as usize]
    }
    pub fn get_constant_mut(&mut self, x: i32) -> &mut Value {
        &mut self.constants_[x as usize]
    }

    pub fn successors_of(&self, bb: u32) -> &[u32] {
        self.cfg.as_ref().expect("CFG not computed").get_succs(&bb)
    }
    pub fn predecessors_of(&self, bb: u32) -> &[u32] {
        self.cfg.as_ref().expect("CFG not computed").get_preds(&bb)
    }
}

impl Traceable for CodeBlock {
    fn trace_with(&self, tracer: &mut Tracer) {
        self.constants
            .iter()
            .for_each(|(_, val)| val.trace_with(tracer));
    }
}

impl Finalizer for CodeBlock {
    fn finalize(&mut self) {
        if let Some(mut cfg) = self.cfg.take() {
            cfg.inner = IndexMap::new();
        }
        self.loopanalysis = None;
        self.code.clear();
        self.constants_.clear();
        self.constants.clear();
    }
}

#[derive(Clone)]
pub struct CFGNode {
    block: u32,
    preds: Vec<u32>,
    succs: Vec<u32>,
}

pub struct CodeCFG {
    inner: IndexMap<u32, CFGNode>,
}

impl CodeCFG {
    fn empty() -> Self {
        Self {
            inner: IndexMap::new(),
        }
    }

    pub fn get_blocks(&self) -> Vec<u32> {
        self.inner.keys().map(|x| x.clone()).collect()
    }

    pub fn get_preds(&self, block: &u32) -> &Vec<u32> {
        &self.inner.get(block).unwrap().preds
    }

    pub fn get_succs(&self, block: &u32) -> &Vec<u32> {
        &self.inner.get(block).unwrap().succs
    }
    pub fn add_pred(&mut self, block: u32, p: u32) {
        self.inner.get_mut(&block).unwrap().preds.push(p);
    }

    pub fn add_succ(&mut self, block: u32, p: u32) {
        self.inner.get_mut(&block).unwrap().succs.push(p);
    }
    pub fn replace_pred(&mut self, block: u32, p: u32, with: u32) {
        *self
            .inner
            .get_mut(&block)
            .unwrap()
            .preds
            .iter_mut()
            .find(|x| **x == p)
            .unwrap() = with;
    }

    pub fn replace_succ(&mut self, block: u32, p: u32, with: u32) {
        *self
            .inner
            .get_mut(&block)
            .unwrap()
            .succs
            .iter_mut()
            .find(|x| **x == p)
            .unwrap() = with;
    }
    pub fn has_edge(&self, from: &u32, to: &u32) -> bool {
        if self.inner.contains_key(from) {
            let ref node = self.inner.get(from).unwrap();
            for succ in node.succs.iter() {
                if succ == to {
                    return true;
                }
            }
        }
        false
    }

    /// checks if there exists a path between from and to, without excluded node
    pub fn has_path_with_node_excluded(&self, from: &u32, to: &u32, exclude_node: &u32) -> bool {
        // we cannot exclude start and end of the path
        assert!(exclude_node != from && exclude_node != to);

        if from == to {
            true
        } else {
            // we are doing BFS

            // visited nodes
            let mut visited: LinkedHashSet<&u32> = LinkedHashSet::new();
            // work queue
            let mut work_list: Vec<&u32> = vec![];
            // initialize visited nodes, and work queue
            visited.insert(from);
            work_list.push(from);

            while !work_list.is_empty() {
                let n = work_list.pop().unwrap();
                for succ in self.get_succs(n) {
                    if succ == exclude_node {
                        // we are not going to follow a path with the excluded
                        // node
                        continue;
                    } else {
                        // if we are reaching destination, return true
                        if succ == to {
                            return true;
                        }

                        // push succ to work list so we will traverse them later
                        if !visited.contains(succ) {
                            visited.insert(succ);
                            work_list.push(succ);
                        }
                    }
                }
            }

            false
        }
    }
}

pub fn build_cfg_for_code(code: &Vec<BasicBlock>) -> CodeCFG {
    let mut ret = CodeCFG::empty();
    let mut predecessors_: LinkedHashMap<u32, LinkedHashSet<u32>> = LinkedHashMap::new();
    for (_id, block) in code.iter().enumerate() {
        if block.code.is_empty() {
            continue;
        }
        for target in block.code.last().unwrap().branch_targets() {
            match predecessors_.get_mut(&target) {
                Some(set) => {
                    set.insert(block.id);
                }
                None => {
                    let mut set = LinkedHashSet::new();
                    set.insert(block.id);
                    predecessors_.insert(target, set);
                }
            }
        }
    }

    let mut successors_: LinkedHashMap<u32, LinkedHashSet<u32>> = LinkedHashMap::new();
    for (_id, block) in code.iter().enumerate() {
        if block.code.is_empty() {
            continue;
        }

        for target in block.code.last().unwrap().branch_targets() {
            match successors_.get_mut(&block.id) {
                Some(set) => {
                    set.insert(target);
                }
                None => {
                    let mut set = LinkedHashSet::new();
                    set.insert(target);
                    successors_.insert(block.id, set);
                }
            }
        }
    }

    for (id, block) in code.iter().enumerate() {
        let mut node = CFGNode {
            block: id as _,
            preds: vec![],
            succs: vec![],
        };
        if predecessors_.contains_key(&block.id) {
            for pred in predecessors_.get(&block.id).unwrap() {
                node.preds.push(*pred as u32)
            }
        }

        if successors_.contains_key(&block.id) {
            for succ in successors_.get(&block.id).unwrap() {
                node.succs.push(*succ as u32);
            }
        }
        ret.inner.insert(block.id, node);
    }

    ret
}
