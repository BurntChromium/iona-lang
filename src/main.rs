#![allow(dead_code)]

use std::env;
use std::error::Error;
use std::fs;
use std::time::Instant;

mod compiler_errors;
mod grammars;
mod lex;
mod parse;
mod properties;

use crate::compiler_errors::{display_problem, ProblemClass};

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging level
    let log_level: ProblemClass = ProblemClass::Lint;
    // Capture command line
    let args: Vec<String> = env::args().collect();
    let file: &str = if args.len() == 1 {
        "main.iona"
    } else {
        &args[1]
    };
    // Try to open linked file
    let maybe_text = fs::read_to_string(file);
    let program_root: String;
    if maybe_text.is_err() {
        return Err(format!("unable to find file {}, aborting compilation", file).into());
    } else {
        program_root = maybe_text.unwrap();
    }
    // Start timer
    let now = Instant::now();
    // Debug: print the file
    // println!("input file is: \n{}", program_root);
    // Lex the file
    let tokens = lex::lex(&program_root);
    // Parse the file
    let (_, errors) = parse::parse(tokens, false);
    let mut okay: bool = true;
    let elapsed = now.elapsed();
    println!("Finished compiling in {:.2?}", elapsed);
    for err in errors {
        if err.class == ProblemClass::Error {
            okay = false;
        }
        if err.class >= log_level {
            display_problem(&program_root, "issue during parsing", err);
        }
    }
    if okay {
        Ok(())
    } else {
        Err("program failed during parsing".into())
    }
}
