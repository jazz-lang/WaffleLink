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

const TRACE_LOOPANALYSIS: bool = true;
use super::*;
use crate::util;
use crate::util::arc::Arc;
use cfg::*;
use hashlink::{LinkedHashMap, LinkedHashSet};
use log::trace;
use std::iter::FromIterator;
use util::multimap::*;
use util::tree::Tree;

pub fn loopanalysis(cf: &mut Arc<Function>, cfg: &FunctionCFG) -> BCLoopAnalysisResult {
    let prologue = 0;
    let dominators = compute_dominators(&cf, &cfg);
    trace!("---dominators---");
    trace!("{:?}", dominators);

    let idoms = compute_immediate_dominators(&dominators);
    trace!("---immediate dominators---");
    trace!("{:?}", idoms);

    let domtree = compute_domtree(prologue.clone(), &idoms);
    trace!("---domtree---");
    trace!("{:?}", domtree);

    let loops = compute_loops(&domtree, &cfg);
    trace!("---loops---");
    trace!("{:?}", loops);

    let merged_loops = compute_merged_loop(&loops);
    trace!("---merged loops---");
    trace!("{:?}", merged_loops);

    let loop_nest_tree = compute_loop_nest_tree(prologue.clone(), &merged_loops);
    trace!("---loop-nest tree---");
    trace!("{:?}", loop_nest_tree);

    let loop_depth = compute_loop_depth(&loop_nest_tree, &merged_loops);
    trace!("---loop depth---");
    trace!("{:?}", loop_depth);
    let result = BCLoopAnalysisResult {
        domtree,
        loops,
        loop_nest_tree,
        loop_depth,
    };

    result
}
pub type BCDomTree = Tree<u16>;
type BCLoopNestTree = Tree<u16>;
#[derive(Debug)]
pub struct BCNaturalLoop {
    header: u16,
    backedge: u16,
    blocks: LinkedHashSet<u16>,
}
#[derive(Debug)]
struct BCMergedLoop {
    header: u16,
    backedges: LinkedHashSet<u16>,
    blocks: LinkedHashSet<u16>,
}
#[allow(dead_code)]
pub struct BCLoopAnalysisResult {
    pub domtree: BCDomTree,
    pub loops: LinkedRepeatableMultiMap<u16, BCNaturalLoop>,
    pub loop_nest_tree: BCLoopNestTree,
    pub loop_depth: LinkedHashMap<u16, usize>,
}

fn compute_dominators(cf: &Arc<Function>, cfg: &FunctionCFG) -> LinkedMultiMap<u16, u16> {
    let mut dominators = LinkedMultiMap::new();
    let all_blocks = {
        let mut ret = LinkedHashSet::new();
        for bb in cf.code.iter() {
            ret.insert(bb.index as u16);
        }
        ret
    };
    let entry = 0u16;
    for block in cf.code.iter() {
        if block.index as u16 != entry {
            dominators.insert_set(block.index as u16, all_blocks.clone());
        }
    }
    dominators.insert(entry, entry);
    let mut work_queue: LinkedHashSet<u16> = LinkedHashSet::new();
    for succ in cfg.get_succs(&entry) {
        work_queue.insert(*succ);
    }
    while let Some(cur) = work_queue.pop_front() {
        let preds = cfg.get_preds(&cur);
        let new_set = {
            let mut intersect = LinkedHashSet::new();
            if dominators.contains_key(&preds[0]) {
                for dp in dominators.get(&preds[0]).unwrap().iter() {
                    intersect.insert(dp.clone());
                }
            }

            for p in preds.iter() {
                let dp_set = dominators.get(p).unwrap();
                intersect.retain(|x| dp_set.contains(x));
            }

            let mut union = LinkedHashSet::new();
            union.insert(cur.clone());
            add_all(&mut union, intersect);
            union
        };
        if new_set == *dominators.get(&cur).unwrap() {
        } else {
            dominators.replace_set(cur.clone(), new_set);
            work_queue.extend(cfg.get_succs(&cur));
        }
    }

    dominators
}

fn compute_immediate_dominators(dominators: &LinkedMultiMap<u16, u16>) -> LinkedHashMap<u16, u16> {
    let mut immediate_doms = LinkedHashMap::new();
    for (n, doms) in dominators.iter() {
        trace_if!(TRACE_LOOPANALYSIS, "compute idom(n={:?})", n);
        for candidate in doms.iter() {
            trace_if!(TRACE_LOOPANALYSIS, "  check candidate {:?}", candidate);
            if candidate != n {
                let mut candidate_is_dom = true;
                for d in doms.iter() {
                    trace_if!(
                        TRACE_LOOPANALYSIS,
                        "  check if {:?} dominates d={:?}",
                        candidate,
                        d
                    );
                    if d != candidate && d != n {
                        if is_dom(candidate, d, &dominators) {
                            trace_if!(
                                TRACE_LOOPANALYSIS,
                                "  failed, as {:?} dominates other dominator {:?}",
                                candidate,
                                d
                            );
                            candidate_is_dom = false;
                        }
                    } else {
                        trace_if!(TRACE_LOOPANALYSIS, "  skip, as d==candidate or d==n");
                    }
                }
                if candidate_is_dom {
                    assert!(!immediate_doms.contains_key(n));
                    trace_if!(TRACE_LOOPANALYSIS, "  add idom({:?}={:?})", n, candidate);
                    immediate_doms.insert(n.clone(), candidate.clone());
                }
            } else {
                trace_if!(TRACE_LOOPANALYSIS, "  skip,candidate is n");
            }
        }
    }
    //assert_eq!(immediate_doms.len(), dominators.len() - 1); // entry block does not have idom.
    immediate_doms
}

fn compute_domtree(entry: u16, idoms: &LinkedHashMap<u16, u16>) -> BCDomTree {
    let mut domtree = BCDomTree::new(entry);
    for (x, idom_x) in idoms.iter() {
        domtree.insert(idom_x.clone(), x.clone());
    }
    domtree
}

fn identify_single_loop(
    header: &u16,
    backedge: &u16,
    nodes: &LinkedHashSet<u16>,
    cfg: &FunctionCFG,
) -> BCNaturalLoop {
    trace_if!(
        TRACE_LOOPANALYSIS,
        "find loop with header {} and backedge {}",
        header,
        backedge
    );
    let mut loop_blocks = LinkedHashSet::new();
    for x in nodes.iter() {
        if x == header || x == backedge {
            loop_blocks.insert(x.clone());
        } else if cfg.has_path_with_node_excluded(x, backedge, header) {
            loop_blocks.insert(x.clone());
        }
    }
    BCNaturalLoop {
        header: *header,
        backedge: *backedge,
        blocks: loop_blocks,
    }
}

fn identify_loop(
    header: &u16,
    domtree: &BCDomTree,
    cfg: &FunctionCFG,
) -> Option<Vec<BCNaturalLoop>> {
    trace_if!(TRACE_LOOPANALYSIS, "find loop with header {}", header);
    let descendants = domtree.get_all_descendants(header);
    trace_if!(TRACE_LOOPANALYSIS, "descendants {:?}", descendants);
    let mut ret = None;
    for n in descendants.iter() {
        if cfg.has_edge(n, header) {
            let lp = identify_single_loop(header, n, &descendants, cfg);
            if ret.is_none() {
                ret = Some(vec![lp]);
            } else {
                ret.as_mut().unwrap().push(lp);
            }
        }
    }
    ret
}

fn compute_loops(
    domtree: &BCDomTree,
    cfg: &FunctionCFG,
) -> LinkedRepeatableMultiMap<u16, BCNaturalLoop> {
    let mut ret = LinkedRepeatableMultiMap::new();
    let mut worklist = vec![domtree.root()];
    while !worklist.is_empty() {
        let cur = worklist.pop().unwrap();
        if let Some(loops) = identify_loop(cur, domtree, cfg) {
            ret.insert_vec(cur.clone(), loops);
        }
        if domtree.has_children(cur) {
            for child in domtree.get_children(cur) {
                worklist.push(child);
            }
        }
    }
    ret
}

fn compute_merged_loop(
    loops: &LinkedRepeatableMultiMap<u16, BCNaturalLoop>,
) -> LinkedHashMap<u16, BCMergedLoop> {
    let mut merged_loops = LinkedHashMap::new();
    for (header, natural_loops) in loops.iter() {
        let mut merged_loop = BCMergedLoop {
            header: *header,
            backedges: LinkedHashSet::new(),
            blocks: LinkedHashSet::new(),
        };
        for l in natural_loops.iter() {
            merged_loop.backedges.insert(l.backedge.clone());
            add_all(&mut merged_loop.blocks, l.blocks.clone());
        }
        merged_loops.insert(*header, merged_loop);
    }
    merged_loops
}

fn compute_loop_nest_tree(
    root: u16,
    merged_loops: &LinkedHashMap<u16, BCMergedLoop>,
) -> BCLoopNestTree {
    trace_if!(TRACE_LOOPANALYSIS, "compute loop-nest tree");
    let mut loop_nest_tree = Tree::new(root);
    for header in merged_loops.keys() {
        trace_if!(TRACE_LOOPANALYSIS, "check loop: {}", header);
        let mut outer_loop_candidate = None;
        let mut outer_loop_size = {
            use std::usize;
            usize::MAX
        };
        for (outer_header, outer_merged_loop) in merged_loops.iter() {
            // nested loop - add an edge from outer loop header to this loop
            // header
            if header != outer_header && outer_merged_loop.blocks.contains(header) {
                let loop_size = outer_merged_loop.blocks.len();
                if loop_size < outer_loop_size {
                    outer_loop_candidate = Some(outer_header);
                    outer_loop_size = loop_size;
                }
            }
        }
        if let Some(outer_loop) = outer_loop_candidate {
            loop_nest_tree.insert(outer_loop.clone(), header.clone());
        } else {
            // this header is not a nested loop - add an edge from root to this
            // loop header
            loop_nest_tree.insert(root.clone(), header.clone());
        }
    }

    loop_nest_tree
}

fn add_all<V: Eq + std::hash::Hash>(x: &mut LinkedHashSet<V>, mut y: LinkedHashSet<V>) {
    while !y.is_empty() {
        let entry = y.pop_front().unwrap();
        x.insert(entry);
    }
}
/// whether a dominates b (i.e. b is one of the dominators of a
fn is_dom(a: &u16, b: &u16, dominators: &LinkedMultiMap<u16, u16>) -> bool {
    dominators.contains_key_val(b, a)
}
fn compute_loop_depth(
    tree: &BCLoopNestTree,
    merged_loops: &LinkedHashMap<u16, BCMergedLoop>,
) -> LinkedHashMap<u16, usize> {
    trace_if!(TRACE_LOOPANALYSIS, "compute loop depth");
    let mut ret = LinkedHashMap::new();
    record_depth(0, tree.root(), tree, merged_loops, &mut ret);
    ret
}

fn record_depth(
    depth: usize,
    node: &u16,
    tree: &BCLoopNestTree,
    merged_loops: &LinkedHashMap<u16, BCMergedLoop>,
    map: &mut LinkedHashMap<u16, usize>,
) {
    // insert the header with the deapth
    trace_if!(TRACE_LOOPANALYSIS, "Header {} = Depth {}", node, depth);
    map.insert(node.clone(), depth);
    // also find all the blocks that belong to the header and are not inner loop
    // header and insert them with the same depth
    if let Some(merged_loop) = merged_loops.get(node) {
        for b in merged_loop.blocks.iter() {
            if !merged_loops.contains_key(b) {
                map.insert(b.clone(), depth);
                trace_if!(TRACE_LOOPANALYSIS, "{} = Depth {}", b, depth);
            }
        }
    }
    if tree.has_children(node) {
        for c in tree.get_children(node).iter() {
            record_depth(depth + 1, c, tree, merged_loops, map);
        }
    }
}