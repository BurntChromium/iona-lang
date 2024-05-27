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
            ProblemClass::Lint => write!(f, "lint"),
            ProblemClass::Warning => write!(f, "earning"),
            ProblemClass::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CompilerProblem {
    class: ProblemClass,
    pub message: String,
    hint: String,
    line: usize,
    word_index: usize,
}

impl CompilerProblem {
    pub fn new(
        class: ProblemClass,
        msg: &str,
        hint: &str,
        line: usize,
        word: usize,
    ) -> CompilerProblem {
        CompilerProblem {
            class,
            message: msg.to_string(),
            hint: hint.to_string(),
            line,
            word_index: word,
        }
    }
}

/// Pass in the raw program text and a compiler problem to print out issues
///
/// The `message_context` is a string written by the caller of the function that errored (So error_message might be: "Problem with a function declaration", and then the actual error message is whatever was returned by the fn)
pub fn display_problem(program_text: &str, message_context: &str, problem: CompilerProblem) {
    // Context is 3 lines: the line above, the problem line, and the line below
    let top_line = problem.line.saturating_sub(2);
    let color_hex_code: &str = match problem.class {
        ProblemClass::Error => "\x1b[1;31m",
        ProblemClass::Warning => "\x1b[1;33m",
        ProblemClass::Lint => "\x1b[1;35m",
    };
    let mut line_number = top_line;
    let context = program_text
        .lines()
        .enumerate()
        .skip(top_line)
        .take(3)
        .map(|(context_index, line)| {
            line_number += 1;
            // Color the middle line (index 1)
            if context_index == 1 {
                format!(
                    "   \x1b[1;34m{line_number} |\x1b[0m {}{}\x1b[0m\n",
                    color_hex_code, line
                )
            } else {
                format!("   \x1b[1;34m{line_number} |\x1b[0m {}\n", line)
            }
        })
        .collect::<String>();

    println!(
        // Hex codes are for colored output
        // We have to push line number up by 1 b/c zero-index vs 1-index
        "{color_hex_code}{}\x1b[0m: {message_context} on line {}: {}\n{}\n\x1b[1;34m hint:\x1b[0m {}",
        problem.class, problem.line+1, problem.message, context.trim_end(), problem.hint
    );
}
