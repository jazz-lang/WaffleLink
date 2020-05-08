pub mod def;
pub mod virtual_reg;
// Register numbers used in bytecode operations have different meaning according to their ranges:
//      0x80000000-0xFFFFFFFF  Negative indices from the CallFrame pointer are entries in the call frame.
//      0x00000000-0x3FFFFFFF  Forwards indices from the CallFrame pointer are local vars and temporaries with the function's callframe.
//      0x40000000-0x7FFFFFFF  Positive indices from 0x40000000 specify entries in the constant pool on the CodeBlock.
pub const FIRST_CONSTNAT_REG_INDEX: i32 = 0x40000000;
use def::*;
use hashlink::{LinkedHashMap, LinkedHashSet};
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
    pub constants: LinkedHashMap<Value, usize>,
    pub arg_regs_count: u32,
    pub tmp_regs_count: u32,
    pub code: Vec<BasicBlock>,
    pub hotness: usize,
    pub cfg: Option<CodeCFG>,
    pub loopanalysis: Option<crate::bytecompiler::loopanalysis::BCLoopAnalysisResult>,
    pub jit_stub:
        Option<extern "C" fn(&mut osr::OSREntry, &mut Runtime, Value, &[Value]) -> JITResult>,
}

impl CodeBlock {
    pub fn dump<W: std::fmt::Write>(&self, b: &mut W) -> std::fmt::Result {
        for bb in self.code.iter() {
            writeln!(b, "%{}: ", bb.id)?;
            for (i, ins) in bb.code.iter().enumerate() {
                writeln!(b, "  [{:04}] {}", i, ins)?;
            }
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
}

impl Traceable for CodeBlock {
    fn trace_with(&self, tracer: &mut Tracer) {
        self.constants
            .iter()
            .for_each(|(_, val)| val.trace_with(tracer));
    }
}

impl Finalizer for CodeBlock {}

#[derive(Clone)]
pub struct CFGNode {
    block: u32,
    preds: Vec<u32>,
    succs: Vec<u32>,
}

pub struct CodeCFG {
    inner: LinkedHashMap<u32, CFGNode>,
}

impl CodeCFG {
    fn empty() -> Self {
        Self {
            inner: LinkedHashMap::new(),
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
