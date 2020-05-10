/// allows coalescing
const COALESCING: bool = true;
/// abort after N rewrite iterations
/// (this is used to detect any possible infinite loop due to bugs)
const MAX_REWRITE_ITERATIONS_ALLOWED: usize = 50;
/// check invariants in every loop
/// (this will make register allocation run extremely slow - be careful
/// when using this with large workloads)
const CHECK_INVARIANTS: bool = false;

use super::interference_graph::*;
use super::loopanalysis::*;
use crate::bytecode::*;
use cgc::api::*;

use hashlink::{linked_hash_map::LinkedHashMap, LinkedHashSet};
use log::error;
use virtual_reg::*;
const VERBOSE: bool = false;
/// GraphColoring algorithm
/// based on Appel's book section 11.4
pub struct GraphColoring {
    pub cf: Handle<CodeBlock>,
    pub ig: InterferenceGraph,

    /// how many coloring iteration have we done?
    /// In case that a bug may trigger the coloring iterate endlessly, we use
    /// this count to stop
    iteration_count: usize,

    /// machine registers, preassigned a color
    precolored: LinkedHashSet<VirtualRegister>,
    /// all colors available
    colors: LinkedHashMap<usize, LinkedHashSet<VirtualRegister>>,
    /// temporaries, not precolored and not yet processed
    initial: LinkedHashSet<VirtualRegister>,

    /// list of low-degree non-move-related nodes
    worklist_simplify: LinkedHashSet<VirtualRegister>,
    /// low-degree move related nodes
    worklist_freeze: LinkedHashSet<VirtualRegister>,
    /// nodes marked for possible spilling during this round
    worklist_spill: LinkedHashSet<VirtualRegister>,
    /// nodes that are selected for spilling, but not yet spilled
    /// (select_spill() is called on it)
    waiting_for_spill: LinkedHashSet<VirtualRegister>,
    /// nodes marked for spilling during this round
    spilled_nodes: LinkedHashSet<VirtualRegister>,
    /// temps that have been coalesced
    /// when u <- v is coalesced, v is added to this set and u put back on some
    /// work list
    coalesced_nodes: LinkedHashSet<VirtualRegister>,
    /// nodes successfully colored
    colored_nodes: LinkedHashSet<VirtualRegister>,
    /// stack containing temporaries removed from the graph
    select_stack: Vec<VirtualRegister>,

    /// moves that have been coalesced
    coalesced_moves: LinkedHashSet<Move>,
    /// moves whose source and target interfere
    constrained_moves: LinkedHashSet<Move>,
    /// moves that will no longer be considered for coalescing
    frozen_moves: LinkedHashSet<Move>,
    /// moves enabled for possible coalescing
    worklist_moves: LinkedHashSet<Move>,
    /// moves not yet ready for coalescing
    active_moves: LinkedHashSet<Move>,

    /// a mapping from a node to the list of moves it is associated with
    movelist: LinkedHashMap<VirtualRegister, LinkedHashSet<Move>>,
    /// when a move (u, v) has been coalesced, and v put in coalescedNodes,
    /// then alias(v) = u
    alias: LinkedHashMap<VirtualRegister, VirtualRegister>,

    /// we need to know the mapping between scratch temp -> original temp
    spill_scratch_temps: LinkedHashMap<VirtualRegister, VirtualRegister>,
}

impl GraphColoring {
    /// starts coloring
    pub fn start(cf: Handle<CodeBlock>, a: &BCLoopAnalysisResult) -> GraphColoring {
        GraphColoring::start_with_spill_history(LinkedHashMap::new(), 0, cf, a)
    }

    /// restarts coloring with spill history
    fn start_with_spill_history(
        spill_scratch_temps: LinkedHashMap<VirtualRegister, VirtualRegister>,
        iteration_count: usize,
        mut cf: Handle<CodeBlock>,
        analysis: &BCLoopAnalysisResult,
    ) -> GraphColoring {
        assert!(
            iteration_count < MAX_REWRITE_ITERATIONS_ALLOWED,
            "reach graph coloring max rewrite iterations ({}), probably something is going wrong",
            MAX_REWRITE_ITERATIONS_ALLOWED
        );
        let iteration_count = iteration_count + 1;

        trace_if!(VERBOSE, "Initializing coloring allocator...");

        let ig = build_interference_graph_chaitin_briggs(&mut cf.get_mut().code, analysis);
        //ig.print();
        /*for bb in cf.code.iter_mut() {
            bb.code.retain(|item| {
                if item.is_final() {
                    return true;
                }

                //item.get_defs().iter().filter(|x| x.is_local()).all(|x| ig.get_degree_of(*x) == 0)
                for item in item.get_defs() {
                    if item.is_local() {
                        if ig.get_degree_of(item) == 0 {
                            continue;
                        } else {
                            return true;
                        }
                    } else {
                        return true;
                    }

                }
                false
            });
        }*/
        let coloring = GraphColoring {
            cf: cf,
            ig: ig,
            iteration_count: iteration_count,
            precolored: LinkedHashSet::new(),
            colors: {
                let mut map = LinkedHashMap::new();
                map.insert(0, LinkedHashSet::new());
                map
            },
            colored_nodes: LinkedHashSet::new(),
            initial: LinkedHashSet::new(),
            worklist_moves: LinkedHashSet::new(),
            movelist: LinkedHashMap::new(),
            active_moves: LinkedHashSet::new(),
            coalesced_nodes: LinkedHashSet::new(),
            coalesced_moves: LinkedHashSet::new(),
            constrained_moves: LinkedHashSet::new(),
            alias: LinkedHashMap::new(),
            worklist_spill: LinkedHashSet::new(),
            waiting_for_spill: LinkedHashSet::new(),
            spilled_nodes: LinkedHashSet::new(),

            spill_scratch_temps: spill_scratch_temps,
            worklist_freeze: LinkedHashSet::new(),
            frozen_moves: LinkedHashSet::new(),
            worklist_simplify: LinkedHashSet::new(),
            select_stack: Vec::new(),
        };

        coloring.regalloc(analysis)
    }

    pub fn spills(&self) -> Vec<VirtualRegister> {
        let mut spills = vec![];

        let spill_count = self.spilled_nodes.len();
        if spill_count > 0 {
            for n in self.spilled_nodes.iter() {
                spills.push(self.ig.get_temp_of(*n));
            }
        }

        spills
    }
    pub fn get_alias(&self, node: VirtualRegister) -> VirtualRegister {
        if self.coalesced_nodes.contains(&node) {
            self.get_alias(*self.alias.get(&node).unwrap())
        } else {
            node
        }
    }

    fn add_worklist(&mut self, node: VirtualRegister) {
        if !self.is_move_related(node)
            && self.ig.get_degree_of(node) < self.n_regs_for_node(node) as u32
        {
            trace_if!(
                VERBOSE,
                "  move {} from worklistFreeze to worklistSimplify",
                node
            );
            self.worklist_freeze.remove(&node);
            self.worklist_simplify.insert(node);
        }
    }

    fn check_ok(&self, u: VirtualRegister, v: VirtualRegister) -> bool {
        for t in self.adjacent(v).iter() {
            let t = *t;
            if !self.ok(t, u) {
                return false;
            }
        }

        true
    }

    fn ok(&self, t: VirtualRegister, r: VirtualRegister) -> bool {
        let degree_t = self.ig.get_degree_of(t);
        let k = self.n_regs_for_node(t) as u32;

        degree_t < k as u32 || self.precolored.contains(&t) || self.ig.is_in_adj_set(t, r)
    }

    fn check_conservative(&self, u: VirtualRegister, v: VirtualRegister) -> bool {
        let adj_u = self.adjacent(u);
        let adj_v = self.adjacent(v);
        let nodes = {
            let mut ret = adj_u;
            add_all(&mut ret, adj_v);
            ret
        };

        let n_regs_for_group = self.n_regs_for_node(u);
        self.conservative(nodes, n_regs_for_group)
    }
    fn simplify(&mut self) {
        // remove next element from worklist_simplify, we know its not empty
        let node = self.worklist_simplify.pop_front().unwrap();

        trace_if!(VERBOSE, "Simplifying {}", node);

        self.select_stack.push(node);
        for m in self.adjacent(node).iter() {
            self.decrement_degree(*m);
        }
    }

    fn adjacent(&self, n: VirtualRegister) -> LinkedHashSet<VirtualRegister> {
        let mut adj = LinkedHashSet::new();

        // add n's successors
        for s in self.ig.get_adj_list(n).iter() {
            adj.insert(*s);
        }

        // removeAll(select_stack)
        for s in self.select_stack.iter() {
            adj.remove(s);
        }

        // removeAll(coalesced_nodes)
        for s in self.coalesced_nodes.iter() {
            adj.remove(s);
        }

        adj
    }
    fn decrement_degree(&mut self, n: VirtualRegister) {
        if self.precolored.contains(&n) {
            return;
        }

        let d = self.ig.get_degree_of(n);
        debug_assert!(d != 0);
        self.ig.set_degree_of(n, d - 1);
        trace_if!(
            VERBOSE,
            "  decrement degree of {} from {} to {}",
            n,
            d,
            d - 1
        );

        if d == self.n_regs_for_node(n) as u32 {
            trace_if!(VERBOSE, "  {}'s degree is K, no longer need to spill it", n);
            let mut nodes = self.adjacent(n);
            nodes.insert(n);
            self.enable_moves(nodes);

            trace_if!(VERBOSE, "  remove {} from worklistSpill", n);
            self.worklist_spill.remove(&n);

            if self.is_move_related(n) {
                trace_if!(VERBOSE, "  {} is move related, push to worklistFreeze", n);
                self.worklist_freeze.insert(n);
            } else {
                trace_if!(
                    VERBOSE,
                    "  {} is not move related, push to worklistSimplify",
                    n
                );
                self.worklist_simplify.insert(n);
            }
        }
    }

    fn coalesce(&mut self) {
        let m = self.worklist_moves.pop_front().unwrap();

        trace_if!(VERBOSE, "Coalescing on {:?}...", m);
        trace_if!(VERBOSE, "  (pop {:?} form worklistMoves)", m);

        let x = self.get_alias(m.from);
        let y = self.get_alias(m.to);
        trace_if!(VERBOSE, "  resolve alias: {} -> {}", m.from, x);
        trace_if!(VERBOSE, "  resolve alias: {} -> {}", m.to, y);

        let (u, v, precolored_u, precolored_v) = {
            if self.precolored.contains(&y) {
                let u = y;
                let v = x;
                let precolored_u = true;
                let precolored_v = self.precolored.contains(&v);

                (u, v, precolored_u, precolored_v)
            } else {
                let u = x;
                let v = y;
                let precolored_u = self.precolored.contains(&u);
                let precolored_v = self.precolored.contains(&v);

                (u, v, precolored_u, precolored_v)
            }
        };
        trace_if!(
            VERBOSE,
            "  u={}, v={}, precolored_u={}, precolroed_v={}",
            u,
            v,
            precolored_u,
            precolored_v
        );
        /*
        // if they are not from the same register group, we cannot coalesce them
        if self.ig.get_group_of(u) != self.ig.get_group_of(v) {
            if !precolored_v {
                self.add_worklist(v);
            }
            if !precolored_u {
                self.add_worklist(u);
            }
            self.constrained_moves.insert(m);
            info!(
                "  u and v are temporaries of different register groups, cannot coalesce: {:?}",
                m
            );
            return;
        }*/

        // if u or v is a machine register that is not usable/not a color, we
        // cannot coalesce
        if precolored_u {
            let group = 0;
            if !self.colors.get(&group).unwrap().contains(&u) {
                if !precolored_v {
                    self.add_worklist(v);
                }
                self.constrained_moves.insert(m);
                trace_if!(
                    VERBOSE,
                    "  u is precolored but not a usable color, cannot coalesce"
                );
                return;
            }
        }
        if precolored_v {
            let group = 0;
            if !self.colors.get(&group).unwrap().contains(&v) {
                if !precolored_u {
                    self.add_worklist(u);
                }
                self.constrained_moves.insert(m);
                trace_if!(
                    VERBOSE,
                    "  v is precolored but not a usable color, cannot coalesce"
                );
                return;
            }
        }

        if u == v {
            trace_if!(VERBOSE, "  u == v, coalesce the move");
            self.coalesced_moves.insert(m);
            if !precolored_u {
                self.add_worklist(u);
            }
        } else if precolored_v || self.ig.is_in_adj_set(u, v) {
            trace_if!(VERBOSE, "  precolored_v: {}", precolored_v);
            trace_if!(VERBOSE, "  is_adj(u, v): {}", self.ig.is_in_adj_set(u, v));
            trace_if!(
                VERBOSE,
                "  v is precolored or u,v is adjacent, the move is constrained"
            );
            self.constrained_moves.insert(m);
            if !precolored_u {
                self.add_worklist(u);
            }
            if !precolored_v {
                self.add_worklist(v);
            }
        } else if (precolored_u && self.check_ok(u, v))
            || (!precolored_u && self.check_conservative(u, v))
        {
            trace_if!(VERBOSE, "  ok(u, v) = {}", self.check_ok(u, v));
            trace_if!(
                VERBOSE,
                "  conservative(u, v) = {}",
                self.check_conservative(u, v)
            );

            trace_if!(
                VERBOSE,
                "  precolored_u&&ok(u,v) || !precolored_u&&conserv(u,v), \
                 coalesce and combine the move"
            );
            self.coalesced_moves.insert(m);
            self.combine(u, v);
            if !precolored_u {
                self.add_worklist(u);
            }
        } else {
            trace_if!(VERBOSE, "  cannot coalesce the move");
            trace_if!(VERBOSE, "  insert {:?} to activeMoves", m);
            self.active_moves.insert(m);
        }
    }
    fn combine(&mut self, u: VirtualRegister, v: VirtualRegister) {
        trace_if!(VERBOSE, "  Combine temps {} and {}...", u, v);
        if self.worklist_freeze.contains(&v) {
            trace_if!(VERBOSE, "  remove {} from worklistFreeze", v);
            self.worklist_freeze.remove(&v);
        } else {
            trace_if!(VERBOSE, "  remove {} from worklistSpill", v);
            self.worklist_spill.remove(&v);
        }
        self.coalesced_nodes.insert(v);

        self.alias.insert(v, u);

        {
            // movelist[u] <- movelist[u] + movelist[v]
            let movelist_v = self.get_movelist(v);

            for m in movelist_v.iter() {
                GraphColoring::add_to_movelist(&mut self.movelist, u, *m)
            }
        }

        let mut nodes = LinkedHashSet::new();
        nodes.insert(v);
        self.enable_moves(nodes);

        for t in self.adjacent(v).iter() {
            let t = *t;
            self.add_edge(t, u);
            self.decrement_degree(t);
        }

        if self.worklist_freeze.contains(&u)
            && self.ig.get_degree_of(u) >= self.n_regs_for_node(u) as u32
        {
            trace_if!(VERBOSE, "  move {} from worklistFreeze to worklistSpill", u);
            self.worklist_freeze.remove(&u);
            self.worklist_spill.insert(u);
        }
    }
    fn add_to_movelist(
        movelist: &mut LinkedHashMap<VirtualRegister, LinkedHashSet<Move>>,
        reg: VirtualRegister,
        mov: Move,
    ) {
        trace_if!(VERBOSE, "  add {:?} to movelist[{}]", mov, reg);
        if movelist.contains_key(&reg) {
            let list = movelist.get_mut(&reg).unwrap();
            list.insert(mov);
        } else {
            let mut list = LinkedHashSet::new();
            list.insert(mov);
            movelist.insert(reg, list);
        }
    }

    fn add_edge(&mut self, u: VirtualRegister, v: VirtualRegister) {
        self.ig.add_edge(u, v);
    }

    fn enable_moves(&mut self, nodes: LinkedHashSet<VirtualRegister>) {
        trace_if!(VERBOSE, "  enable moves of: {:?}", nodes);
        for n in nodes.iter() {
            let n = *n;
            for mov in self.node_moves(n).iter() {
                let mov = *mov;
                if self.active_moves.contains(&mov) {
                    trace_if!(
                        VERBOSE,
                        "  move {:?} from activeMoves to worklistMoves",
                        mov
                    );
                    self.active_moves.remove(&mov);
                    self.worklist_moves.insert(mov);
                }
            }
        }
    }

    fn conservative(&self, nodes: LinkedHashSet<VirtualRegister>, n_regs_for_group: usize) -> bool {
        let mut k = 0;
        for n in nodes.iter() {
            // TODO: do we check if n is precolored?
            if self.precolored.contains(n)
                || (self.ig.get_degree_of(*n) as usize) >= n_regs_for_group
            {
                k += 1;
            }
        }
        k < n_regs_for_group
    }

    fn n_regs_for_node(&self, _node: VirtualRegister) -> usize {
        255
    }
    fn make_work_list(&mut self) {
        trace_if!(VERBOSE, "Making work list from initials...");
        while !self.initial.is_empty() {
            let node = self.initial.pop_front().unwrap();

            // degree >= K
            if self.ig.get_degree_of(node) >= self.n_regs_for_node(node) as u32 {
                trace_if!(
                    VERBOSE,
                    "  {} 's degree >= reg number limit (K), push to worklistSpill",
                    node
                );
                self.worklist_spill.insert(node);
            } else if self.is_move_related(node) {
                trace_if!(
                    VERBOSE,
                    "  {} is move related, push to worklistFreeze",
                    node
                );
                self.worklist_freeze.insert(node);
            } else {
                trace_if!(
                    VERBOSE,
                    "  {} has small degree and not move related, push to worklistSimplify",
                    node
                );
                self.worklist_simplify.insert(node);
            }
        }
    }
    fn is_move_related(&mut self, node: VirtualRegister) -> bool {
        !self.node_moves(node).is_empty()
    }

    fn is_spillable(&self, temp: VirtualRegister) -> bool {
        // if a temporary is created as scratch temp for a spilled temporary, we
        // should not spill it again (infinite loop otherwise)
        if self.spill_scratch_temps.contains_key(&temp) {
            false
        } else {
            true
        }
    }

    fn node_moves(&mut self, node: VirtualRegister) -> LinkedHashSet<Move> {
        let mut moves = LinkedHashSet::new();

        // addAll(active_moves)
        for m in self.active_moves.iter() {
            moves.insert(m.clone());
        }

        // addAll(worklist_moves)
        for m in self.worklist_moves.iter() {
            moves.insert(m.clone());
        }

        let mut retained = LinkedHashSet::new();
        let movelist = self.get_movelist(node);
        for m in moves.iter() {
            if movelist.contains(m) {
                retained.insert(*m);
            }
        }

        retained
    }
    fn get_movelist(&self, reg: VirtualRegister) -> LinkedHashSet<Move> {
        if let Some(list) = self.movelist.get(&reg) {
            list.clone()
        } else {
            LinkedHashSet::new()
        }
    }

    pub fn get_assignments(&self) -> LinkedHashMap<VirtualRegister, VirtualRegister> {
        let mut ret = LinkedHashMap::new();

        for node in self.ig.nodes() {
            let temp = self.ig.get_temp_of(node);

            if temp.to_local() < 256 {
                continue;
            } else {
                let alias = self.get_alias(node);
                let machine_reg = match self.ig.get_color_of(alias) {
                    Some(reg) => reg,
                    None => panic!(
                        "Reg{}/{:?} (aliased as Reg{}/{:?}) is not assigned with a color",
                        self.ig.get_temp_of(node),
                        node,
                        self.ig.get_temp_of(alias),
                        alias
                    ),
                };

                ret.insert(temp, machine_reg);
            }
        }

        ret
    }

    pub fn get_spill_scratch_temps(&self) -> LinkedHashMap<VirtualRegister, VirtualRegister> {
        self.spill_scratch_temps.clone()
    }

    fn freeze(&mut self) {
        // it is not empty (checked before)
        let node = self.freeze_heuristics();
        trace_if!(VERBOSE, "Freezing {}...", node);

        trace_if!(
            VERBOSE,
            "  move {} from worklistFreeze to worklistSimplify",
            node
        );
        self.worklist_freeze.remove(&node);
        self.worklist_simplify.insert(node);
        self.freeze_moves(node);
    }

    fn freeze_heuristics(&mut self) -> VirtualRegister {
        use std::f32;
        // we try to freeze a node that appears less frequently
        // we compute freeze cost based on all the moves related with this node
        let mut candidate = None;
        let mut candidate_cost = f32::MAX;
        for &n in self.worklist_freeze.iter() {
            let freeze_cost = self.ig.get_freeze_cost(n);

            if freeze_cost < candidate_cost {
                candidate = Some(n);
                candidate_cost = freeze_cost;
            }
        }

        assert!(candidate.is_some());
        candidate.unwrap()
    }

    fn freeze_moves(&mut self, u: VirtualRegister) {
        trace_if!(VERBOSE, "  freeze moves for {}", u);
        for m in self.node_moves(u).iter() {
            let m = *m;
            //            let mut v = self.get_alias(m.from);
            //            if v == self.get_alias(u) {
            //                v = self.get_alias(m.to);
            //            }
            let x = m.from;
            let y = m.to;
            let v = if self.get_alias(y) == self.get_alias(u) {
                self.get_alias(x)
            } else {
                self.get_alias(y)
            };

            trace_if!(VERBOSE, "  move {:?} from activeMoves to frozenMoves", m);
            self.active_moves.remove(&m);
            self.frozen_moves.insert(m);

            //            if !self.precolored.contains(&v) &&
            // self.node_moves(v).is_empty() &&
            // self.ig.get_degree_of(v) < self.n_regs_for_node(v)
            if self.worklist_freeze.contains(&v) && self.node_moves(v).is_empty() {
                trace_if!(
                    VERBOSE,
                    "  move {} from worklistFreeze to worklistSimplify",
                    v
                );
                self.worklist_freeze.remove(&v);
                self.worklist_simplify.insert(v);
            }
        }
    }

    fn select_spill(&mut self) {
        trace_if!(VERBOSE, "Selecting a node to spill...");
        let mut m: Option<VirtualRegister> = None;

        for n in self.worklist_spill.iter() {
            let n = *n;
            // if a node is not spillable, we guarantee that we do not spill it
            if !self.is_spillable(n) {
                trace_if!(VERBOSE, "  {} is not spillable", n);
                continue;
            }

            if m.is_none() {
                trace_if!(VERBOSE, "  {} is the initial choice", n);
                m = Some(n);
            } else {
                let cur_m = m.unwrap();
                let cost_m = self.ig.get_spill_cost(cur_m);
                let cost_n = self.ig.get_spill_cost(n);
                if cost_n < cost_m {
                    trace_if!(VERBOSE, "  {} is preferred: ({} < {})", n, cost_n, cost_m);
                    m = Some(n);
                }
            }
        }

        // m is not none
        assert!(m.is_some(), "failed to select any node to spill");
        let m = m.unwrap();
        trace_if!(VERBOSE, "  Spilling {}...", m);
        trace_if!(
            VERBOSE,
            "  move {:?} from worklistSpill to worklistSimplify",
            m
        );
        self.waiting_for_spill.insert(m);
        self.worklist_spill.remove(&m);
        self.worklist_simplify.insert(m);
        self.freeze_moves(m);
    }

    fn assign_colors(&mut self) {
        trace_if!(VERBOSE, "---coloring done---");

        let mut coloring_queue: Vec<VirtualRegister> = self.coloring_queue_heuristic();
        while !coloring_queue.is_empty() {
            let n = coloring_queue.pop().unwrap();
            trace_if!(VERBOSE, "Assigning color to {}", n);

            let mut ok_colors: LinkedHashSet<VirtualRegister> =
                self.colors.get(&0).unwrap().clone();

            trace_if!(VERBOSE, "  all the colors for this temp: {:?}", ok_colors);

            for w in self.ig.get_adj_list(n).iter() {
                let w_alias = self.get_alias(*w);
                match self.ig.get_color_of(w_alias) {
                    None => {} // do nothing
                    Some(color) => {
                        trace_if!(
                            VERBOSE,
                            "  color {} is used for its neighbor {:?} (aliasing to {:?})",
                            color,
                            w,
                            w_alias
                        );
                        ok_colors.remove(&color);
                    }
                }
            }
            trace_if!(VERBOSE, "  available colors: {:?}", ok_colors);

            if ok_colors.is_empty() {
                trace_if!(VERBOSE, "  {} is a spilled node", n);
                self.spilled_nodes.insert(n);
            } else {
                let color = self.color_heuristic(n, &mut ok_colors);
                trace_if!(VERBOSE, "  Color {} as {}", n, color);

                self.colored_nodes.insert(n);
                self.ig.color_node(n, color);
            }
        }

        for n in self.coalesced_nodes.iter() {
            let n = *n;
            let alias = self.get_alias(n);
            if let Some(alias_color) = self.ig.get_color_of(alias) {
                trace_if!(
                    VERBOSE,
                    "  Assign color to {} based on aliased {}",
                    n,
                    alias
                );
                trace_if!(VERBOSE, "  Color {} as {}", n, alias_color);
                self.ig.color_node(n, alias_color);
            }
        }
    }

    //    /// we pick colors for node that has higher weight (higher spill cost)
    //    fn coloring_queue_heuristic(&self) -> Vec<ID> {
    //        let mut ret = self.select_stack.clone();
    //        ret.sort_by_key(|x| self.ig.get_spill_cost(*x));
    //        ret.reverse();
    //        ret
    //    }
    fn build(&mut self) {
        if COALESCING {
            trace_if!(VERBOSE, "Coalescing enabled, build move list...");
            let ref ig = self.ig;
            for m in ig.moves().iter() {
                trace_if!(VERBOSE, "  add {:?} to worklistMoves", m);
                self.worklist_moves.insert(*m);
                GraphColoring::add_to_movelist(&mut self.movelist, m.from, *m);
                GraphColoring::add_to_movelist(&mut self.movelist, m.to, *m);
            }
        } else {
            trace_if!(VERBOSE, "Coalescing disabled...");
        }

        trace_if!(VERBOSE, "Build freeze cost for each node...");
        // we try to avoid freeze a node that is involved in many moves
        for n in self.ig.nodes() {
            // freeze_cost(n) = SUM ((spill_cost(src) + spill_cost(dst)) for m
            // (mov src->dst) in movelist[n])
            let closure = {
                let mut ret = LinkedHashSet::new();
                let mut worklist = LinkedHashSet::new();
                worklist.insert(n);

                while !worklist.is_empty() {
                    let n = worklist.pop_front().unwrap();
                    for m in self.get_movelist(n).iter() {
                        if !ret.contains(&m.from) {
                            ret.insert(m.from);
                            worklist.insert(m.from);
                        }
                        if !ret.contains(&m.to) {
                            ret.insert(m.to);
                            worklist.insert(m.to);
                        }
                    }
                }

                ret
            };

            let mut freeze_cost = 0f32;
            for related_node in closure.iter() {
                freeze_cost += self.ig.get_spill_cost(*related_node);
            }

            self.ig.set_freeze_cost(n, freeze_cost);
            trace_if!(VERBOSE, "  {} closure: {:?}", n, closure);
            trace_if!(VERBOSE, "     freeze cost = {}", freeze_cost);
        }
    }

    fn coloring_queue_heuristic(&self) -> Vec<VirtualRegister> {
        self.select_stack.clone()
    }

    /// we favor choosing colors that will make any frozen moves able to be
    /// eliminated
    fn color_heuristic(
        &self,
        reg: VirtualRegister,
        available_colors: &mut LinkedHashSet<VirtualRegister>,
    ) -> VirtualRegister {
        trace_if!(
            VERBOSE,
            "  Find color for {} in {:?}",
            reg,
            available_colors
        );

        // we use spill cost as weight.
        // A node that has higher spill cost is used more frequently, and has a
        // higher weight we favor choosing color that has a higher
        // weight
        let mut candidate_weight: LinkedHashMap<VirtualRegister, f32> = LinkedHashMap::new();

        for mov in self.frozen_moves.iter() {
            // find the other part of the mov
            let other = if mov.from == reg { mov.to } else { mov.from };
            let alias = self.get_alias(other);
            let other_color = self.ig.get_color_of(alias);
            let other_weight = self.ig.get_spill_cost(alias);
            // if the other part is colored and that color is available,
            // we will favor the choice of the color
            if let Some(other_color) = other_color {
                if available_colors.contains(&other_color) {
                    let total_weight = if candidate_weight.contains_key(&other_color) {
                        candidate_weight.get(&other_color).unwrap() + other_weight
                    } else {
                        other_weight
                    };
                    candidate_weight.insert(other_color, total_weight);
                    trace_if!(
                        VERBOSE,
                        "    favor {} to eliminate {:?} (weight={})",
                        other_color,
                        mov,
                        total_weight
                    );
                }
            }
        }

        if candidate_weight.is_empty() {
            trace_if!(VERBOSE, "    no candidate, use first avaiable color");
            available_colors.pop_front().unwrap()
        } else {
            let mut c = None;
            let mut c_weight = 0f32;
            for (&id, &weight) in candidate_weight.iter() {
                if c.is_none() || (c.is_some() && c_weight < weight) {
                    c = Some(id);
                    c_weight = weight;
                }
            }
            assert!(c.is_some());
            let color = c.unwrap();
            assert!(available_colors.contains(&color));
            trace_if!(VERBOSE, "    pick candidate of most weight: {}", color);
            color
        }
    }

    /// does coloring register allocation
    fn regalloc(mut self, a: &BCLoopAnalysisResult) -> GraphColoring {
        trace_if!(VERBOSE, "---InterenceGraph---");
        self.ig.print();

        // precolor for all machine registers
        for reg in 0..256 {
            let reg_id = VirtualRegister::tmp(reg);
            self.precolored.insert(reg_id);
        }

        // put usable registers as available colors
        for reg in 0..256 {
            let reg_id = VirtualRegister::tmp(reg);
            let group = 0;
            self.colors.get_mut(&group).unwrap().insert(reg_id);
        }

        // push uncolored nodes to initial work set
        for node in self.ig.nodes() {
            if !self.ig.is_colored(node) {
                self.initial.insert(node);
            }
        }

        // initialize work
        self.build();
        self.make_work_list();

        // main loop
        while {
            if !self.worklist_simplify.is_empty() {
                self.simplify();
            } else if !self.worklist_moves.is_empty() {
                self.coalesce();
            } else if !self.worklist_freeze.is_empty() {
                self.freeze();
            } else if !self.worklist_spill.is_empty() {
                self.select_spill();
            }

            if CHECK_INVARIANTS {
                self.check_invariants();
            }

            !(self.worklist_simplify.is_empty()
                && self.worklist_moves.is_empty()
                && self.worklist_freeze.is_empty()
                && self.worklist_spill.is_empty())
        } {}

        // pick color for nodes
        self.assign_colors();

        // finish
        // if we need to spill
        if !self.spilled_nodes.is_empty() {
            trace_if!(VERBOSE, "spill required");
            if cfg!(debug_assertions) {
                trace_if!(VERBOSE, "nodes to be spilled:");
                for node in self.spilled_nodes.iter() {
                    trace_if!(VERBOSE, "{}", *node);
                }
            }

            // rewrite program to insert spilling code
            self.rewrite_program();

            // recursively redo graph coloring
            return GraphColoring::start_with_spill_history(
                self.spill_scratch_temps.clone(),
                self.iteration_count,
                self.cf,
                a,
            );
        }

        self
    }

    fn check_invariants(&self) {
        self.checkinv_degree();
        self.checkinv_simplify_worklist();
        self.checkinv_freeze_worklist();
        self.checkinv_spill_worklist();
    }
    fn rewrite_program(&mut self) {
        unimplemented!("Spilling is not yet supported");
        /*let spills = self.spills();
        let mut spilled_mem = LinkedHashMap::new();
        // allocating frame slots for every spilled temp
        for reg_id in spills.iter() {
            let mem = self
                .cf
                .frame
                .alloc_slot_for_spilling(ssa_entry.value().clone(), self.vm);
            spilled_mem.insert(*reg_id, mem.clone());
        }
        let scratch_temps = backend::spill_rewrite(&spilled_mem, self.func, self.cf, self.vm);
        for (k, v) in scratch_temps {
            self.spill_scratch_temps.insert(k, v);
        }*/
    }

    fn checkinv_degree(&self) {
        trace_if!(VERBOSE, "Check invariant: degree...");
        // degree invariant
        // for u in simplifyWorklist \/ freezeWorklist \/ spillWorklist =>
        //   degree(u) = |adjList(u) /\
        //               (precolored \/ simplifyWorklist \/ freezeWorklist \/
        // spillWorklist)|

        let union = {
            let mut ret = LinkedHashSet::new();
            add_all(&mut ret, self.worklist_simplify.clone());
            add_all(&mut ret, self.worklist_freeze.clone());
            add_all(&mut ret, self.worklist_spill.clone());
            ret
        };

        for u in union.iter() {
            let degree_u = self.ig.get_degree_of(*u);

            let set: LinkedHashSet<VirtualRegister> = {
                let adj: LinkedHashSet<VirtualRegister> = self.ig.get_adj_list(*u).clone();

                let mut union_: LinkedHashSet<VirtualRegister> = LinkedHashSet::new();
                add_all(&mut union_, self.precolored.clone());
                add_all(&mut union_, union.clone());

                {
                    let mut intersect: LinkedHashSet<VirtualRegister> = LinkedHashSet::new();
                    for item in adj.iter() {
                        if union_.contains(item) {
                            intersect.insert(*item);
                        }
                    }
                    intersect
                }
            };

            self.checkinv_assert(
                degree_u == set.len() as u32,
                format!("degree({})={} != set(len)={}", u, degree_u, set.len()),
            );
        }
    }

    fn checkinv_simplify_worklist(&self) {
        trace_if!(VERBOSE, "Check invariant: worklistSimplify...");
        // for u in simplifyWorkList
        //   either u is 'selected for spilling', otherwise
        //   degree(u) < K &&
        //   movelist(u) /\ (activeMoves \/ worklistMoves) = {} (emtpy)
        for u in self.worklist_simplify.iter() {
            if self.waiting_for_spill.contains(&u) {
                // no longer needs to check
                return;
            } else {
                // 1st cond: degree(u) < K
                let degree = self.ig.get_degree_of(*u);
                self.checkinv_assert(
                    degree < self.n_regs_for_node(*u) as u32,
                    format!("degree({})={} < K fails", u, degree),
                );

                // 2nd cond
                let movelist = self.get_movelist(*u);
                let union = {
                    let mut ret = self.active_moves.clone();
                    add_all(&mut ret, self.worklist_moves.clone());
                    ret
                };
                let intersect = {
                    let mut ret = LinkedHashSet::new();
                    for m in movelist.iter() {
                        if union.contains(m) {
                            ret.insert(*m);
                        }
                    }
                    ret
                };

                self.checkinv_assert(
                    intersect.len() == 0,
                    format!("intersect({}) is not empty", u),
                );
            }
        }
    }

    fn checkinv_freeze_worklist(&self) {
        trace_if!(VERBOSE, "Check invariant: worklistFreeze...");
        // for u in freezeWorklist
        //   degree(u) < K &&
        //   moveList(u) /\ (activeMoves \/ worklistMoves) != {} (non empty)
        for u in self.worklist_freeze.iter() {
            // 1st cond: degree(u) < K
            let degree = self.ig.get_degree_of(*u);
            self.checkinv_assert(
                degree < self.n_regs_for_node(*u) as u32,
                format!("degree({})={} < K fails", u, degree),
            );

            // 2nd cond
            // 2nd cond
            let movelist = self.get_movelist(*u);
            let union = {
                let mut ret = self.active_moves.clone();
                add_all(&mut ret, self.worklist_moves.clone());
                ret
            };
            let intersect = {
                let mut ret = LinkedHashSet::new();
                for m in movelist.iter() {
                    if union.contains(m) {
                        ret.insert(*m);
                    }
                }
                ret
            };
            self.checkinv_assert(intersect.len() != 0, format!("intersect({}) is empty", u));
        }
    }

    fn checkinv_spill_worklist(&self) {
        trace_if!(VERBOSE, "Check invariant: worklistSpill...");
        // for u in spillWorklist
        //    degree(u) >= K
        for u in self.worklist_spill.iter() {
            let degree = self.ig.get_degree_of(*u);
            self.checkinv_assert(
                degree >= self.n_regs_for_node(*u) as u32,
                format!("degree({})={} >= K fails", u, degree),
            );
        }
    }

    fn checkinv_assert(&self, cond: bool, msg: String) {
        if !cond {
            error!("{}", msg);

            // dump current state
            trace_if!(VERBOSE, "Current state:");

            trace_if!(VERBOSE, "simplifyWorklist: {:?}", self.worklist_simplify);
            trace_if!(VERBOSE, "freezeWorklist: {:?}", self.worklist_freeze);
            trace_if!(VERBOSE, "spillWorklist: {:?}", self.worklist_spill);
            trace_if!(VERBOSE, "worklistMoves: {:?}", self.worklist_moves);

            for node in self.ig.nodes() {
                trace_if!(
                    VERBOSE,
                    "Node {}: degree={}",
                    node,
                    self.ig.get_degree_of(node)
                );
                trace_if!(VERBOSE, "         adjList={:?}", self.ig.get_adj_list(node));
                trace_if!(VERBOSE, "         moveList={:?}", self.get_movelist(node));
            }

            panic!()
        }
    }
}

fn add_all<V: Eq + std::hash::Hash>(x: &mut LinkedHashSet<V>, mut y: LinkedHashSet<V>) {
    while !y.is_empty() {
        let entry = y.pop_front().unwrap();
        x.insert(entry);
    }
}
