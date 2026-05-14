use crate::types::{Cnf, Lit};
use std::io::{self, BufRead};

#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    InvalidHeader,
    InvalidLiteral(String),
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        ParseError::Io(e)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(e) => write!(f, "IO error: {e}"),
            ParseError::InvalidHeader => write!(f, "invalid DIMACS header"),
            ParseError::InvalidLiteral(s) => write!(f, "invalid literal: {s}"),
        }
    }
}

pub fn parse_dimacs(reader: impl BufRead) -> Result<Cnf, ParseError> {
    let mut num_vars = 0u32;
    let mut _num_clauses = 0u32;
    let mut clauses = Vec::new();
    let mut current_clause = Vec::new();
    let mut header_seen = false;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() || line.starts_with('c') {
            continue;
        }

        if line.starts_with('p') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 4 || parts[1] != "cnf" {
                return Err(ParseError::InvalidHeader);
            }
            num_vars = parts[2]
                .parse()
                .map_err(|_| ParseError::InvalidHeader)?;
            _num_clauses = parts[3]
                .parse()
                .map_err(|_| ParseError::InvalidHeader)?;
            header_seen = true;
            continue;
        }

        if !header_seen {
            return Err(ParseError::InvalidHeader);
        }

        for token in line.split_whitespace() {
            let val: i32 = token
                .parse()
                .map_err(|_| ParseError::InvalidLiteral(token.to_string()))?;

            if val == 0 {
                clauses.push(std::mem::take(&mut current_clause));
            } else {
                let var = val.unsigned_abs();
                let neg = val < 0;
                current_clause.push(Lit::new(var, neg));
            }
        }
    }

    // Handle case where file doesn't end with 0
    if !current_clause.is_empty() {
        clauses.push(current_clause);
    }

    Ok(Cnf::new(num_vars, clauses))
}