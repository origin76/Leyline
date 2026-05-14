use leyline::parser::parse_dimacs;
use leyline::solver::{solve, SolveResult};
use std::io::BufReader;

fn solve_str(input: &str) -> SolveResult {
    let cnf = parse_dimacs(BufReader::new(input.as_bytes())).unwrap();
    solve(cnf)
}

#[test]
fn sat_simple() {
    assert_eq!(solve_str("p cnf 2 2\n1 2 0\n-1 2 0\n"), SolveResult::Sat);
}

#[test]
fn unsat_simple() {
    assert_eq!(solve_str("p cnf 1 2\n1 0\n-1 0\n"), SolveResult::Unsat);
}

#[test]
fn sat_unit_propagation() {
    assert_eq!(solve_str("p cnf 2 2\n1 0\n-1 -2 0\n"), SolveResult::Sat);
}

#[test]
fn sat_empty_formula() {
    assert_eq!(solve_str("p cnf 0 0\n"), SolveResult::Sat);
}

#[test]
fn unsat_empty_clause() {
    assert_eq!(solve_str("p cnf 1 1\n0\n"), SolveResult::Unsat);
}

#[test]
fn sat_three_vars() {
    assert_eq!(
        solve_str("p cnf 3 3\n1 2 0\n-1 3 0\n-2 -3 0\n"),
        SolveResult::Sat
    );
}

#[test]
fn unsat_pigeonhole_2_3() {
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
