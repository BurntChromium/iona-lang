//! Tooling for reporting compilation errors

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
