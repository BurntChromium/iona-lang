//! Tooling for reporting compilation errors

use std::cmp::max;
use std::fmt::Display;

#[derive(Debug, Eq, PartialEq)]
pub enum ProblemClass {
    Lint,
    Warning,
    Error,
}

impl Display for ProblemClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProblemClass::Lint => write!(f, "Lint"),
            ProblemClass::Warning => write!(f, "Warning"),
            ProblemClass::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CompilerProblem {
    class: ProblemClass,
    message: String,
    line: usize,
    word_index: usize,
}

impl CompilerProblem {
    pub fn new(class: ProblemClass, msg: &str, line: usize, word: usize) -> CompilerProblem {
        CompilerProblem {
            class,
            message: msg.to_string(),
            line,
            word_index: word,
        }
    }
}

/// Pass in the raw program text and a compiler problem to print out issues
/// 
/// The `message_context` is a string written by the caller of the function that errored (So error_message might be: "Problem with a function declaration", and then the actual error message is whatever was returned by the fn)
pub fn display_problem(input_text: &str, message_context: &str, problem: CompilerProblem) {
    // Context is 3 lines: the line above, the problem line, and the line below
    let top_line = max(problem.line - 2, 0);
    let context = input_text
        .lines()
        .skip(top_line)
        .take(3)
        .collect::<String>();
    println!(
        "{message_context} on line {}: {}. Code context:\n{context}\n",
        problem.line, problem.message
    );
}
