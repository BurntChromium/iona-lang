#![allow(dead_code)]

use std::env;
use std::error::Error;
use std::fs;
use std::time::Instant;

mod codegen_c;
mod compiler_errors;
mod grammars;
mod lex;
mod parse;
mod permissions;
mod properties;

use crate::parse::{compute_scopes, populate_function_table};
use compiler_errors::{display_problem, CompilerProblem, ProblemClass};

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
    let (mut nodes, mut errors) = parse::parse(tokens);
    let elapsed = now.elapsed();
    println!("Finished compiling in {:.2?}", elapsed);
    // Do post-processing on the AST -- just stick all errors onto the parse list and print all at once
    // 1) Compute scopes (we MUST do this before trying to build function table)
    errors.extend(compute_scopes(&mut nodes));
    // 2) Build a function table
    let function_table = populate_function_table(&nodes);
    if function_table.is_err() {
        errors.extend(function_table.unwrap_err());
    }
    // Display parsing errors
    let okay = display_error_list(&program_root, errors, log_level);
    // Final output
    if okay {
        Ok(())
    } else {
        Err("program failed during parsing".into())
    }
}

fn display_error_list(
    program_text: &str,
    errors: Vec<CompilerProblem>,
    log_level: ProblemClass,
) -> bool {
    let mut okay = true;
    for err in errors {
        if err.class == ProblemClass::Error {
            okay = false;
        }
        if err.class >= log_level {
            display_problem(&program_text, "issue during parsing", err);
        }
    }
    okay
}
