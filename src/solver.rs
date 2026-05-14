use crate::types::{ClauseId, Cnf, Lit, Var};

struct WatchedClause {
    lits: Vec<Lit>,
    watched: [usize; 2],
}

/// Reason for a propagation: the clause that forced the assignment,
/// and the specific literal in that clause that became true.
type Reason = (ClauseId, Lit);

pub struct Solver {
    clauses: Vec<WatchedClause>,
    watches: Vec<Vec<ClauseId>>,
    assignment: Vec<Option<bool>>,
    trail: Vec<Var>,
    levels: Vec<usize>,
    var_level: Vec<usize>,
    reason: Vec<Option<Reason>>,
    has_empty_clause: bool,
}

#[inline]
fn lit_idx(lit: Lit) -> usize {
    lit.var as usize * 2 + lit.neg as usize
}

impl Solver {
    pub fn new(cnf: Cnf) -> Self {
        let num_vars = cnf.num_vars as usize;
        let mut watches = vec![Vec::new(); (num_vars + 1) * 2];
        let mut clauses = Vec::with_capacity(cnf.clauses.len());
        let mut has_empty_clause = false;

        for lits in cnf.clauses {
            if lits.is_empty() {
                has_empty_clause = true;
                continue;
            }
            let cid = clauses.len() as ClauseId;
            let w1 = 0;
            let w2 = if lits.len() > 1 { 1 } else { 0 };
            watches[lit_idx(lits[w1])].push(cid);
            if w2 != w1 {
                watches[lit_idx(lits[w2])].push(cid);
            }
            clauses.push(WatchedClause {
                lits,
                watched: [w1, w2],
            });
        }

        Self {
            clauses,
            watches,
            assignment: vec![None; num_vars + 1],
            trail: Vec::new(),
            levels: Vec::new(),
            var_level: vec![0; num_vars + 1],
            reason: vec![None; num_vars + 1],
            has_empty_clause,
        }
    }

    fn assign(&mut self, var: Var, value: bool, reason: Option<Reason>) {
        debug_assert!(self.assignment[var as usize].is_none());
        self.assignment[var as usize] = Some(value);
        self.var_level[var as usize] = self.levels.len();
        self.reason[var as usize] = reason;
        self.trail.push(var);
    }

    fn unassign_to_level(&mut self, level: usize) {
        let trail_start = self.levels[level];
        while self.trail.len() > trail_start {
            let var = self.trail.pop().unwrap();
            self.assignment[var as usize] = None;
            self.reason[var as usize] = None;
        }
        self.levels.truncate(level);
    }

    fn is_false(&self, lit: Lit) -> bool {
        match self.assignment[lit.var as usize] {
            Some(val) => !(if lit.neg { !val } else { val }),
            None => false,
        }
    }

    fn is_true(&self, lit: Lit) -> bool {
        match self.assignment[lit.var as usize] {
            Some(val) => if lit.neg { !val } else { val },
            None => false,
        }
    }

    fn propagate(&mut self) -> Option<ClauseId> {
        let mut prop_queue: Vec<Lit> = Vec::new();

        if let Some(&var) = self.trail.last()
            && let Some(val) = self.assignment[var as usize]
        {
            prop_queue.push(Lit::new(var, val));
        }

        while let Some(falsified) = prop_queue.pop() {
            let fidx = lit_idx(falsified);
            let watcher_list = std::mem::take(&mut self.watches[fidx]);
            let mut keep = Vec::new();
            let mut conflict: Option<ClauseId> = None;

            for &cid in &watcher_list {
                let clause = &self.clauses[cid as usize];
                let w0 = clause.watched[0];
                let w1 = clause.watched[1];

                let (first_idx, other_idx) = if clause.lits[w0] == falsified {
                    (w0, w1)
                } else {
                    debug_assert_eq!(clause.lits[w1], falsified);
                    (w1, w0)
                };
                let other_lit = clause.lits[other_idx];

                if self.is_true(other_lit) {
                    keep.push(cid);
                    continue;
                }

                let mut new_watch = None;
                for (i, &lit) in clause.lits.iter().enumerate() {
                    if i == first_idx || i == other_idx {
                        continue;
                    }
                    if !self.is_false(lit) {
                        new_watch = Some(i);
                        break;
                    }
                }

                if let Some(nw_idx) = new_watch {
                    self.clauses[cid as usize].watched = if first_idx == w0 {
                        [nw_idx, other_idx]
                    } else {
                        [other_idx, nw_idx]
                    };
                    let new_lit = self.clauses[cid as usize].lits[nw_idx];
                    self.watches[lit_idx(new_lit)].push(cid);
                } else {
                    keep.push(cid);
                    if self.is_false(other_lit) {
                        if conflict.is_none() {
                            conflict = Some(cid);
                        }
                    } else {
                        let value = !other_lit.neg;
                        self.assign(other_lit.var, value, Some((cid, other_lit)));
                        prop_queue.push(other_lit.negate());
                    }
                }
            }

            self.watches[fidx] = keep;

            if conflict.is_some() {
                return conflict;
            }
        }

        None
    }

    /// 1-UIP conflict analysis via resolution.
    /// Returns (learned_clause, backtrack_level).
    fn analyze_conflict(&mut self, conflict_cid: ClauseId) -> (Vec<Lit>, usize) {
        let current_level = self.levels.len();

        // Collect literals from the conflict clause.
        let mut learned: Vec<Lit> = self.clauses[conflict_cid as usize].lits.clone();

        // Resolve until exactly one literal remains at the current level.
        loop {
            let mut count_at_level = 0;
            for lit in &learned {
                if self.var_level[lit.var as usize] == current_level {
                    count_at_level += 1;
                }
            }

            if count_at_level <= 1 {
                break;
            }

            // Find the most recently assigned variable at the current level.
            let mut resolve_var = 0;
            for &var in self.trail.iter().rev() {
                if self.var_level[var as usize] == current_level
                    && learned.iter().any(|l| l.var == var)
                {
                    resolve_var = var;
                    break;
                }
            }
            debug_assert!(resolve_var > 0);

            // Resolve with the reason clause.
            let (reason_cid, _reason_lit) = self.reason[resolve_var as usize].unwrap();
            let reason_lits = self.clauses[reason_cid as usize].lits.clone();

            let var = resolve_var;
            learned.retain(|l| l.var != var);
            for lit in &reason_lits {
                if lit.var != var && !learned.iter().any(|l| l.var == lit.var) {
                    learned.push(*lit);
                }
            }
        }

        // Compute backtrack level: second-highest decision level in the learned clause.
        let mut bt_level = 0usize;
        for lit in &learned {
            let lv = self.var_level[lit.var as usize];
            if lv != current_level && lv > bt_level {
                bt_level = lv;
            }
        }

        (learned, bt_level)
    }

    fn decide(&self) -> Option<Var> {
        (1..=self.assignment.len() as u32 - 1).find(|&var| self.assignment[var as usize].is_none())
    }

    pub fn solve(&mut self) -> SolveResult {
        if self.has_empty_clause {
            return SolveResult::Unsat;
        }

        loop {
            if let Some(conflict_cid) = self.propagate() {
                if self.levels.is_empty() {
                    return SolveResult::Unsat;
                }

                let (learned, bt_level) = self.analyze_conflict(conflict_cid);

                self.unassign_to_level(bt_level);

                if learned.is_empty() {
                    return SolveResult::Unsat;
                }

                // Add the learned clause and set up watches.
                let cid = self.clauses.len() as ClauseId;
                let w1 = 0;
                let w2 = if learned.len() > 1 { 1 } else { 0 };
                self.watches[lit_idx(learned[w1])].push(cid);
                if w2 != w1 {
                    self.watches[lit_idx(learned[w2])].push(cid);
                }
                self.clauses.push(WatchedClause {
                    lits: learned,
                    watched: [w1, w2],
                });

                // Propagate the unit literal from the learned clause.
                // After backtracking, exactly one literal is unassigned.
                let propagated = self.clauses.last().unwrap().lits.iter().find(|lit| {
                    self.assignment[lit.var as usize].is_none()
                });
                if let Some(&lit) = propagated {
                    let value = !lit.neg;
                    self.assign(lit.var, value, Some((cid, lit)));
                }
            } else {
                match self.decide() {
                    None => return SolveResult::Sat,
                    Some(var) => {
                        self.levels.push(self.trail.len());
                        self.assign(var, true, None);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveResult {
    Sat,
    Unsat,
}

pub fn solve(cnf: Cnf) -> SolveResult {
    let mut solver = Solver::new(cnf);
    solver.solve()
}
