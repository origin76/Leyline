use crate::types::{Cnf, Lit, Var};

/// A clause that's been partially evaluated under the current assignment.
struct EvalClause<'a> {
    lits: &'a [Lit],
}

enum ClauseStatus {
    Sat,
    Conflict,
    Unit(Lit),
    Unresolved,
}

impl<'a> EvalClause<'a> {
    fn evaluate(&self, assignment: &[Option<bool>]) -> ClauseStatus {
        let mut unresolved_lit = None;

        for &lit in self.lits {
            match assignment[lit.var as usize] {
                Some(val) => {
                    let lit_val = if lit.neg { !val } else { val };
                    if lit_val {
                        return ClauseStatus::Sat;
                    }
                }
                None => {
                    if unresolved_lit.is_some() {
                        return ClauseStatus::Unresolved; // 2+ unassigned
                    }
                    unresolved_lit = Some(lit);
                }
            }
        }

        match unresolved_lit {
            Some(lit) => ClauseStatus::Unit(lit),
            None => ClauseStatus::Conflict,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveResult {
    Sat,
    Unsat,
}

pub struct Solver {
    num_vars: u32,
    assignment: Vec<Option<bool>>, // indexed by Var (1-indexed)
    trail: Vec<Var>,               // assignment order, for undo
    levels: Vec<usize>,            // trail index where each decision level starts
}

impl Solver {
    pub fn new(num_vars: u32) -> Self {
        Self {
            num_vars,
            assignment: vec![None; num_vars as usize + 1],
            trail: Vec::new(),
            levels: Vec::new(),
        }
    }

    fn assign(&mut self, var: Var, value: bool) {
        debug_assert!(self.assignment[var as usize].is_none());
        self.assignment[var as usize] = Some(value);
        self.trail.push(var);
    }

    fn unassign_to_level(&mut self, level: usize) {
        let trail_start = self.levels[level];
        while self.trail.len() > trail_start {
            let var = self.trail.pop().unwrap();
            self.assignment[var as usize] = None;
        }
        self.levels.truncate(level);
    }

    fn current_level(&self) -> usize {
        self.levels.len()
    }

    /// Run unit propagation. Returns the level at which to backtrack
    /// if a conflict is found, or None if no conflict.
    fn propagate(&mut self, cnf: &Cnf) -> Option<usize> {
        loop {
            let mut propagated = false;
            let mut conflict_level = None;

            for clause in &cnf.clauses {
                let eval = EvalClause { lits: clause };
                match eval.evaluate(&self.assignment) {
                    ClauseStatus::Sat => continue,
                    ClauseStatus::Unresolved => continue,
                    ClauseStatus::Conflict => {
                        // Conflict: backtrack level is one less than current
                        let level = self.current_level();
                        if level == 0 {
                            return Some(0); // UNSAT at root
                        }
                        conflict_level = Some(level - 1);
                    }
                    ClauseStatus::Unit(lit) => {
                        let value = !lit.neg;
                        self.assign(lit.var, value);
                        propagated = true;
                    }
                }
            }

            if let Some(level) = conflict_level {
                return Some(level);
            }

            if !propagated {
                return None; // fixpoint reached, no conflict
            }
        }
    }

    /// Pick the next unassigned variable (naive: first unassigned).
    fn decide(&self) -> Option<Var> {
        (1..=self.num_vars).find(|&var| self.assignment[var as usize].is_none())
    }

    pub fn solve(&mut self, cnf: &Cnf) -> SolveResult {
        // The "decisions" stack: each entry is (var, tried_positive)
        // We only push to this when making a new decision.
        // On backtrack we pop and try the other polarity.
        let mut decisions: Vec<(Var, bool)> = Vec::new();

        loop {
            // Unit propagate
            if let Some(bt_level) = self.propagate(cnf) {
                // Conflict during propagation
                if bt_level == 0 {
                    return SolveResult::Unsat;
                }
                // Undo to bt_level
                self.unassign_to_level(bt_level);
                // Retry with opposite polarity of the decision at bt_level
                let (var, tried_pos) = decisions.pop().unwrap();
                debug_assert_eq!(self.levels.len(), bt_level);
                if tried_pos {
                    // We tried positive, now try negative
                    self.levels.push(self.trail.len());
                    self.assign(var, false);
                    decisions.push((var, false));
                } else {
                    // Both polarities tried, backtrack further
                    continue; // loop will hit conflict again and backtrack
                }
            } else {
                // No conflict, check if all assigned
                match self.decide() {
                    None => return SolveResult::Sat,
                    Some(var) => {
                        // Make a decision: try positive first
                        self.levels.push(self.trail.len());
                        self.assign(var, true);
                        decisions.push((var, true));
                    }
                }
            }
        }
    }
}

pub fn solve(cnf: &Cnf) -> SolveResult {
    let mut solver = Solver::new(cnf.num_vars);
    solver.solve(cnf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_dimacs;
    use std::io::BufReader;

    fn solve_str(input: &str) -> SolveResult {
        let cnf = parse_dimacs(BufReader::new(input.as_bytes())).unwrap();
        solve(&cnf)
    }

    #[test]
    fn sat_simple() {
        let result = solve_str("p cnf 2 2\n1 2 0\n-1 2 0\n");
        assert_eq!(result, SolveResult::Sat);
    }

    #[test]
    fn unsat_simple() {
        let result = solve_str("p cnf 1 2\n1 0\n-1 0\n");
        assert_eq!(result, SolveResult::Unsat);
    }

    #[test]
    fn sat_unit_propagation() {
        // Unit clause forces x1=true, then x2 must be false
        let result = solve_str("p cnf 2 2\n1 0\n-1 -2 0\n");
        assert_eq!(result, SolveResult::Sat);
    }

    #[test]
    fn sat_empty_formula() {
        let result = solve_str("p cnf 0 0\n");
        assert_eq!(result, SolveResult::Sat);
    }

    #[test]
    fn unsat_empty_clause() {
        // An empty clause (from "0" alone) is always false
        let result = solve_str("p cnf 1 1\n0\n");
        assert_eq!(result, SolveResult::Unsat);
    }

    #[test]
    fn sat_three_vars() {
        let result = solve_str("p cnf 3 3\n1 2 0\n-1 3 0\n-2 -3 0\n");
        assert_eq!(result, SolveResult::Sat);
    }

    #[test]
    fn unsat_pigeonhole_2_3() {
        // 2 holes, 3 pigeons — UNSAT
        // p1=1, p2=2, p3=3 means pigeon in hole 1
        // p4=4, p5=5, p6=6 means pigeon in hole 2
        // Each pigeon must be in some hole
        // Each hole has at most one pigeon
        let input = "p cnf 6 9
1 4 0
2 5 0
3 6 0
-1 -2 0
-1 -3 0
-2 -3 0
-4 -5 0
-4 -6 0
-5 -6 0
";
        assert_eq!(solve_str(input), SolveResult::Unsat);
    }
}
