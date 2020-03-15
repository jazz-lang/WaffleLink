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

use super::*;
use crate::util::arc::*;
use basicblock::*;
use hashlink::{LinkedHashMap, LinkedHashSet};
use instruction::*;
#[derive(Clone)]
pub struct CFGNode {
    block: u16,
    preds: Vec<u16>,
    succs: Vec<u16>,
}

#[derive(Clone)]
pub struct FunctionCFG {
    inner: LinkedHashMap<u16, CFGNode>,
}

impl FunctionCFG {
    fn empty() -> Self {
        Self {
            inner: LinkedHashMap::new(),
        }
    }

    pub fn get_blocks(&self) -> Vec<u16> {
        self.inner.keys().map(|x| x.clone()).collect()
    }

    pub fn get_preds(&self, block: &u16) -> &Vec<u16> {
        &self.inner.get(block).unwrap().preds
    }

    pub fn get_succs(&self, block: &u16) -> &Vec<u16> {
        &self.inner.get(block).unwrap().succs
    }

    pub fn has_edge(&self, from: &u16, to: &u16) -> bool {
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
    pub fn has_path_with_node_excluded(&self, from: &u16, to: &u16, exclude_node: &u16) -> bool {
        // we cannot exclude start and end of the path
        assert!(exclude_node != from && exclude_node != to);

        if from == to {
            true
        } else {
            // we are doing BFS

            // visited nodes
            let mut visited: LinkedHashSet<&u16> = LinkedHashSet::new();
            // work queue
            let mut work_list: Vec<&u16> = vec![];
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

pub fn build_cfg_for_code(code: &Vec<BasicBlock>) -> FunctionCFG {
    let mut ret = FunctionCFG::empty();
    let mut predecessors_: LinkedHashMap<usize, LinkedHashSet<usize>> = LinkedHashMap::new();
    for (id, block) in code.iter().enumerate() {
        if block.instructions.is_empty() {
            continue;
        }
        for target in block.instructions.last().unwrap().branch_targets() {
            match predecessors_.get_mut(&target) {
                Some(set) => {
                    set.insert(id);
                }
                None => {
                    let mut set = LinkedHashSet::new();
                    set.insert(id);
                    predecessors_.insert(target, set);
                }
            }
        }
    }

    let mut successors_: LinkedHashMap<usize, LinkedHashSet<usize>> = LinkedHashMap::new();
    for (id, block) in code.iter().enumerate() {
        if block.instructions.is_empty() {
            continue;
        }

        for target in block.instructions.last().unwrap().branch_targets() {
            match successors_.get_mut(&id) {
                Some(set) => {
                    set.insert(target);
                }
                None => {
                    let mut set = LinkedHashSet::new();
                    set.insert(target);
                    successors_.insert(id, set);
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
        if predecessors_.contains_key(&id) {
            for pred in predecessors_.get(&id).unwrap() {
                node.preds.push(*pred as u16)
            }
        }

        if successors_.contains_key(&id) {
            for succ in successors_.get(&id).unwrap() {
                node.succs.push(*succ as u16);
            }
        }
        ret.inner.insert(id as u16, node);
    }

    ret
}
