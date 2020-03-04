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
use crate::bytecode;
use crate::util::arc::Arc;
use std::collections::{BTreeSet, HashSet};
/// Simplifies CFG, removes unused blocks and replaces some branches.
/// SimpliyCFGPass is really nice in reducing bytecode size, through it doesn't improve performance.
pub struct SimplifyCFGPass;

impl SimplifyCFGPass {
    fn simplify(&self, code: &mut Arc<Vec<BasicBlock>>) {
        if code.len() == 0 {
            return;
        }
        let n_basic_blocks = code.len();
        let mut out_edges: Vec<HashSet<u16>> = vec![HashSet::new(); n_basic_blocks];
        let mut in_edges: Vec<HashSet<u16>> = vec![HashSet::new(); n_basic_blocks];
        for i in 0..n_basic_blocks as u16 {
            let (a, b) = code[i as usize].branch_targets();
            if let Some(v) = a {
                out_edges[i as usize].insert(v);
                in_edges[v as usize].insert(i);
            }
            if let Some(v) = b {
                out_edges[i as usize].insert(v);
                in_edges[v as usize].insert(i);
            }
        }

        for i in 0..n_basic_blocks as u16 {
            if out_edges[i as usize].len() == 1 {
                let j = *out_edges[i as usize].iter().nth(0).unwrap() as u16;
                if in_edges[j as usize].len() == 1 {
                    if *in_edges[j as usize].iter().nth(0).unwrap() == i {
                        out_edges.swap(i as usize, j as usize);
                        out_edges[j as usize].clear();
                        in_edges[j as usize].clear();
                        let v =
                            ::std::mem::replace(&mut code[j as usize], BasicBlock::new(vec![], 0));
                        code[i as usize].join(v);
                    }
                }
            }
        }

        let mut dfs_stack: Vec<usize> = Vec::new();
        let mut dfs_visited: Vec<bool> = vec![false; n_basic_blocks];

        dfs_visited[0] = true;
        dfs_stack.push(0);

        while !dfs_stack.is_empty() {
            let current = dfs_stack.pop().unwrap();

            for other in &out_edges[current] {
                if !dfs_visited[*other as usize] {
                    dfs_visited[*other as usize] = true;
                    dfs_stack.push(*other as usize);
                }
            }
        }

        // collect unused blocks
        {
            let unused_blocks: BTreeSet<usize> =
                (0..code.len()).filter(|i| !dfs_visited[*i]).collect();
            let mut tail = n_basic_blocks - 1;
            let mut remap_list: Vec<(usize, usize)> = Vec::new(); // (to, from)
            for id in &unused_blocks {
                while tail > *id {
                    if unused_blocks.contains(&tail) {
                        tail -= 1;
                    } else {
                        break;
                    }
                }

                // Implies tail > 0
                if tail <= *id {
                    break;
                }

                // Now `id` is the first unused block and `tail`
                // is the last used block
                // Let's exchange them
                remap_list.push((*id, tail));
                code.swap(*id, tail);
                tail -= 1;
            }
            while code.len() > tail + 1 {
                code.pop().unwrap();
            }
            for (to, from) in remap_list {
                for bb in code.iter_mut() {
                    let replaced = bb.try_replace_branch_targets(to as _, from as _);
                    if replaced {}
                }
            }
            //n_basic_blocks = f.basic_blocks.len();
        }
    }
}

impl BytecodePass for SimplifyCFGPass {
    fn execute(&mut self, f: &mut Arc<Vec<BasicBlock>>) {
        self.simplify(f);
    }
}
