pub type Var = u32;
pub type ClauseId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lit {
    pub var: Var,
    pub neg: bool,
}

impl Lit {
    pub fn new(var: Var, neg: bool) -> Self {
        debug_assert!(var > 0, "DIMACS variables are 1-indexed");
        Self { var, neg }
    }

    pub fn positive(var: Var) -> Self {
        Self::new(var, false)
    }

    pub fn negative(var: Var) -> Self {
        Self::new(var, true)
    }

    pub fn negate(self) -> Self {
        Self {
            var: self.var,
            neg: !self.neg,
        }
    }
}

impl std::fmt::Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.neg {
            write!(f, "-{}", self.var)
        } else {
            write!(f, "{}", self.var)
        }
    }
}

pub type Clause = Vec<Lit>;

pub struct Cnf {
    pub num_vars: u32,
    pub clauses: Vec<Clause>,
}

impl Cnf {
    pub fn new(num_vars: u32, clauses: Vec<Clause>) -> Self {
        Self { num_vars, clauses }
    }
}
