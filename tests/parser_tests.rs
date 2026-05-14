use leyline::parser::parse_dimacs;
use leyline::types::Lit;
use std::io::BufReader;

fn parse_str(input: &str) -> leyline::types::Cnf {
    parse_dimacs(BufReader::new(input.as_bytes())).unwrap()
}

#[test]
fn parse_simple() {
    let cnf = parse_str("p cnf 3 2\n1 -2 0\n2 3 0\n");
    assert_eq!(cnf.num_vars, 3);
    assert_eq!(cnf.clauses.len(), 2);
    assert_eq!(cnf.clauses[0], vec![Lit::positive(1), Lit::negative(2)]);
    assert_eq!(cnf.clauses[1], vec![Lit::positive(2), Lit::positive(3)]);
}

#[test]
fn parse_comments() {
    let cnf = parse_str("c this is a comment\np cnf 2 1\n1 2 0\n");
    assert_eq!(cnf.num_vars, 2);
    assert_eq!(cnf.clauses.len(), 1);
}

#[test]
fn parse_multiclause_line() {
    let cnf = parse_str("p cnf 2 2\n1 0 2 0\n");
    assert_eq!(cnf.clauses.len(), 2);
}

#[test]
fn parse_no_trailing_zero() {
    let cnf = parse_str("p cnf 1 1\n1");
    assert_eq!(cnf.clauses.len(), 1);
}
