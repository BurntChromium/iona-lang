//! Parse constructs an abstract syntax tree or equivalent
//!
//! Organizational note: the arrangements of permissible token sequences are defined by the `grammar` crate. Each line of Iona code corresponds to one singular Grammar, and can be parsed into that grammar independently.
//!
//! We represent our AST as a flat list of `Nodes`, and each `Node` is assigned a Grammar and some metadata.

use std::fmt::Debug;

use crate::compiler_errors::CompilerProblem;
use crate::grammars::{
    self, Grammar, GrammarFunctionDeclaration, GrammarImports, GrammarProperties, GrammarReturns,
    GrammarVariableAssignments,
};
use crate::lex::{Symbol, Token};

/// Nodes are objects corresponding to an IR, and each node has exactly one type (each line of code has one effect, or "role" to play).
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
pub enum NodeType {
    FunctionDeclaration,
    PropertyDeclaration,
    ContractDeclaration,
    VariableAssignment,
    TypeDeclaration,
    EffectualFunctionInvocation,
    ImportStatement,
    ReturnStatement,
}

/// Primitive data types (i.e. types not held in a container or struct)
#[derive(Debug, PartialEq, Eq)]
pub enum PrimitiveDataType {
    Void,
    Int,
    Float,
    Str,
    Bool,
}

pub trait Data: Debug {}

#[derive(Debug)]
pub struct Node {
    pub node_type: NodeType,
    pub grammar: Box<dyn Grammar>,
    pub source_line: usize,
}

impl Node {
    pub fn new(node_type: NodeType, grammar: Box<dyn Grammar>, source_line: usize) -> Node {
        Node {
            node_type,
            grammar,
            source_line,
        }
    }
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub data_type: PrimitiveDataType,
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
    let mut nodes = Vec::<Node>::new();
    let mut error_list: Vec<CompilerProblem> = Vec::<CompilerProblem>::new();
    // We will be skipping the iterator from inside the loop, so we do something a little weird looking
    let mut iterator = tokens.iter();
    // At the beginning of each line, apply a grammar to that line
    while let Some(token) = iterator.next() {
        // On a match, grab all tokens in the same line
        // Map the appropriate grammar to that line of tokens, and accumulate any errors
        let mut grammar: Box<dyn Grammar> = match token.symbol {
            // Handle imports
            Symbol::Import => Box::new(GrammarImports::new()),
            // Handle function declaration
            Symbol::FunctionDeclare => Box::new(GrammarFunctionDeclaration::new()),
            // Handle property declarations
            Symbol::PropertyDeclaration => Box::new(GrammarProperties::new()),
            // Handle variable declarations
            Symbol::Set | Symbol::Let => Box::new(GrammarVariableAssignments::new(
                if token.symbol == Symbol::Let {
                    grammars::AssignmentTypes::Initialize
                } else {
                    grammars::AssignmentTypes::Mutate
                },
            )),
            // Handle contracts
            Symbol::ContractPre | Symbol::ContractPost | Symbol::ContractInvariant => {
                Box::new(GrammarFunctionDeclaration::new())
            }
            // Handle return statements
            Symbol::Return => Box::new(GrammarReturns::new()),
            _ => Box::new(GrammarFunctionDeclaration::new()),
        };
        let mut errors: Vec<Option<CompilerProblem>> = Vec::new();
        let future = iterator.clone().peekable();
        for t in future {
            if t.line == token.line {
                errors.push(grammar.step(t));
            } else {
                break;
            }
        }
        // Then force the iterator to catch up
        iterator.nth(errors.len());
        // Check for errors (this happens after skip because consumes iterator)
        let mut okay = true;
        for e in errors {
            if let Some(problem) = e {
                error_list.push(problem);
                okay = false;
            }
        }
        if okay {
            nodes.push(Node::new(
                NodeType::FunctionDeclaration,
                grammar,
                token.line,
            ));
        }
    }
    // Return or provide a list of errors
    if error_list.is_empty() {
        Ok(nodes)
    } else {
        Err(error_list)
    }
}
