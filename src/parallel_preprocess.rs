//! Fused Lex-and-Parsing steps with support for parallel processing

use rayon::prelude::*;

use crate::compiler_errors::CompilerProblem;
use crate::lex::{lex_line, Token};
use crate::parse::{parse, Node};

/// Lexes and parses each line in parallel
pub fn fused_lex_and_parse(input: &str) -> (Vec<Node>, Vec<CompilerProblem>) {
    let lines_list = input.lines().collect::<Vec<&str>>();
    // Use parallel iterator and fold to accumulate results 
    lines_list
        .into_par_iter()
        .enumerate()
        .map(|(line_index, line)| {
            let mut tokens: Vec<Token> = Vec::new();
            lex_line(line, line_index, &mut tokens);
            parse(tokens, true)
        })
        .fold(
            || (Vec::new(), Vec::new()), 
            |(mut acc_nodes, mut acc_problems), (nodes, problems)| {
                acc_nodes.extend(nodes);
                acc_problems.extend(problems);
                (acc_nodes, acc_problems)
            },
        )
        .reduce(
            || (Vec::new(), Vec::new()), 
            |(mut acc_nodes, mut acc_problems), (nodes, problems)| {
                acc_nodes.extend(nodes);
                acc_problems.extend(problems);
                (acc_nodes, acc_problems)
            },
        )
}

// -------------------- Unit Tests --------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lex;
    
    #[test]
    fn compare_single_multi_threaded_1() {
        // This test fails -- not going to look into it now
        let line: &str = "import a b from c";
        let parsed_mt = fused_lex_and_parse(line);
        let lexed = lex(line);
        let parsed_st = parse(lexed, false);
        assert_eq!(parsed_mt.0.len(), parsed_st.0.len());
        assert!(parsed_mt.0.len() > 1);
        assert_eq!(parsed_mt.0[0].node_type, parsed_st.0[0].node_type);
        assert_eq!(parsed_mt.1, parsed_st.1);
    }
}