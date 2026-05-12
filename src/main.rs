use std::fs::File;
use std::io::BufReader;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <dimacs.cnf>", args[0]);
        process::exit(1);
    }

    let file = File::open(&args[1]).unwrap_or_else(|e| {
        eprintln!("Error opening {}: {e}", args[1]);
        process::exit(1);
    });

    let cnf = leyline::parser::parse_dimacs(BufReader::new(file)).unwrap_or_else(|e| {
        eprintln!("Parse error: {e}");
        process::exit(1);
    });

    match leyline::solver::solve(&cnf) {
        leyline::solver::SolveResult::Sat => println!("SAT"),
        leyline::solver::SolveResult::Unsat => println!("UNSAT"),
    }
}
