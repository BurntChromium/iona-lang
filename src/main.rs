#![allow(dead_code)]

use std::env;
use std::error::Error;
use std::fs;

mod lex;
mod parse;

fn main() -> Result<(), Box<dyn Error>> {
    // Capture command line
    let args: Vec<String> = env::args().collect();
    let file: &str;
    if args.len() == 1 {
        file = "main.iona";
    } else {
        file = &args[1];
    }
    // Try to open linked file
    let maybe_text = fs::read_to_string(file);
    let program_root: String;
    if maybe_text.is_err() {
        return Err(format!("unable to find file {}, aborting compilation", file).into());
    } else {
        program_root = maybe_text.unwrap();
    }
    // Debug: print the file
    println!("input file is: \n{}", program_root);
    // Lex the file
    let _ = lex::lex(&program_root);
    return Ok(());
}
