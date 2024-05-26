//! Parse constructs an abstract syntax tree or equivalent
//!
//! Organizational note: the syntax of permissible tokens is defined by the `grammar` crate.

use std::fmt::Debug;

use crate::compiler_errors::CompilerProblem;
use crate::grammars::{Grammar, GrammarFunctionDeclaration};
use crate::lex::{Symbol, Token};

/// Object represents something that has been parsed
///
/// Permissible Nodes
///
/// - FunctionDeclaration: a function declaration is its name and type signature
/// - PropertyDeclaration: a list of properties required by the function
/// - ContractDeclaration: some runtime behavior the fn must obey
/// - VariableAssignment: initializing or changing a variable with `let` / `set`
/// - FunctionInvocation: calling some function
/// - TypeDeclaration: creating a new type
/// - EffectualFunctionInvocation: some fn call without let/set/return (i.e. it exists only for whatever side effect is triggered by calling it)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeType {
    FunctionDeclaration,
    PropertyDeclaration,
    ContractDeclaration,
    VariableAssignment,
    TypeDeclaration,
    EffectualFunctionInvocation,
    ImportStatement,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DataType {
    Void,
    Int,
    Float,
    Str,
    Bool,
}

pub trait Data: Debug {}

pub struct Node {
    node_type: NodeType,
    grammar: Grammar,
    source_line: usize,
    source_string: String,
    children: Vec<Node>,
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub data_type: DataType,
    pub value: Option<Box<dyn Data>>,
}

/// Parse a list of tokens
///
/// Basic idea
/// - On a new line, identify the appropriate grammar
/// - Take all tokens within that line
/// - For each token in the line, feed it through the grammar
/// - Along the way, accumulate any errors we find
/// - When all lines have been mapped, return an Error if we have found any problems, or return a list of nodes if we have not
pub fn parse(tokens: Vec<Token>) -> Result<Vec<Node>, Vec<CompilerProblem>> {
    let nodes = Vec::<Node>::new();
    let mut error_list: Vec<CompilerProblem> = Vec::<CompilerProblem>::new();
    // We will be skipping the iterator from inside the loop, so we do something a little weird looking
    let mut iterator = tokens.iter();
    // At the beginning of each line, apply a grammar to that line
    while let Some(token) = iterator.next() {
        // On a match, grab all tokens in the same line
        // Map the appropriate grammar to that line of tokens, and accumulate any errors
        match token.symbol {
            Symbol::Import => {}
            Symbol::FunctionDeclare => {
                let mut grammar = GrammarFunctionDeclaration::new();
                let mut errors: Vec<Option<CompilerProblem>> = Vec::new();
                let _ = iterator
                    .by_ref()
                    .skip(1)
                    .take_while(|t| t.line == token.line)
                    .map(|t| errors.push(grammar.step(t)));
                if errors.iter().any(|x| x.is_some()) {
                    let _ = errors
                        .into_iter()
                        .filter(|e| e.is_some())
                        .map(|e| error_list.push(e.unwrap()));
                }
            }
            Symbol::PropertyDeclaration => {}
            Symbol::ContractPre | Symbol::ContractPost | Symbol::ContractInvariant => {}
            _ => {}
        }
    }
    // Return or provide a list of errors
    if error_list.is_empty() {
        Ok(nodes)
    } else {
        Err(error_list)
    }
}
