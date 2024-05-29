//! Parse constructs an abstract syntax tree or equivalent
//!
//! Organizational note: the arrangements of permissible token sequences are defined by the `grammar` crate. Each line of Iona code corresponds to one singular Grammar, and can be parsed into that grammar independently.
//!
//! We represent our AST as a flat list of `Nodes`, and each `Node` is assigned a Grammar and some metadata.

use std::fmt::Debug;

use crate::compiler_errors::CompilerProblem;
use crate::grammars::{
    self, FunctionAnnotations, Grammar, GrammarEmpty, GrammarFnAnnotation,
    GrammarFunctionDeclaration, GrammarImports, GrammarReturns, GrammarVariableAssignments,
};
use crate::lex::{Symbol, Token};
use crate::properties::Properties;

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
    FunctionDeclaration,         // done
    PropertyDeclaration,         // done
    PermissionsDeclaration,      // done
    ContractDeclaration,         // TODO
    VariableAssignment,          // done
    TypeDeclaration,             // newtype, TODO
    EffectualFunctionInvocation, // TODO
    ImportStatement,             // done
    ReturnStatement,             // done
    CloseScope,                  // done
    Empty,                       // done
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
/// ### Parameters
///
/// - tokens: a list of tokens from the lexer
/// - fused_mode: if true, assume the input is tokens from a single line (i.e. operating in the "fused-lex-and-parse" mode), and if false, then assume the input is from a whole file or set of files
///
/// Basic idea
/// - On a new line, identify the appropriate grammar
/// - Take all tokens within that line
/// - For each token in the line, feed it through the grammar
/// - Along the way, accumulate any errors we find
/// - When all lines have been mapped, return all nodes and all errors and let the caller decide what to do with it (otherwise, we would swallow warnings and lints)
pub fn parse(tokens: Vec<Token>) -> (Vec<Node>, Vec<CompilerProblem>) {
    let mut nodes = Vec::<Node>::new();
    let mut error_list: Vec<CompilerProblem> = Vec::<CompilerProblem>::new();
    // We will be skipping the iterator from inside the loop, so we do something a little weird looking
    let mut iterator = tokens.iter();
    // At the beginning of each line, apply a grammar to that line
    while let Some(token) = iterator.next() {
        let node_type: NodeType;
        // On a match, grab all tokens in the same line
        // Map the appropriate grammar to that line of tokens, and accumulate any errors
        let mut grammar: Box<dyn Grammar> = match token.symbol {
            // Handle imports
            Symbol::Import => {
                node_type = NodeType::ImportStatement;
                Box::new(GrammarImports::new())
            }
            // Handle function declaration
            Symbol::FunctionDeclare => {
                node_type = NodeType::FunctionDeclaration;
                Box::new(GrammarFunctionDeclaration::new())
            }
            // Handle property declarations (pass in dummy value to signal type)
            Symbol::PropertyDeclaration => {
                node_type = NodeType::PropertyDeclaration;
                Box::new(GrammarFnAnnotation::new(FunctionAnnotations::Prop(
                    Properties::Pure,
                )))
            }
            // Handle permissions declarations (pass in dummy value to signal type)
            Symbol::PermissionsDeclaration => {
                node_type = NodeType::PermissionsDeclaration;
                Box::new(GrammarFnAnnotation::new(FunctionAnnotations::Perm(
                    crate::permissions::Permissions::Custom,
                )))
            }
            // Handle variable declarations
            Symbol::Set | Symbol::Let => {
                node_type = NodeType::VariableAssignment;
                Box::new(GrammarVariableAssignments::new(
                    if token.symbol == Symbol::Let {
                        grammars::AssignmentTypes::Initialize
                    } else {
                        grammars::AssignmentTypes::Mutate
                    },
                ))
            }
            // Handle contracts
            Symbol::ContractPre | Symbol::ContractPost | Symbol::ContractInvariant => {
                node_type = NodeType::ContractDeclaration;
                Box::new(GrammarFunctionDeclaration::new())
            }
            // Handle return statements
            Symbol::Return => {
                node_type = NodeType::ReturnStatement;
                Box::new(GrammarReturns::new())
            }
            // Handle scope closes
            Symbol::BraceClose => {
                node_type = NodeType::CloseScope;
                Box::new(GrammarEmpty::new())
            }
            // Skip comments
            _ => {
                node_type = NodeType::Empty;
                Box::new(GrammarEmpty::new())
            }
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
        // Then force the iterator to catch up (if NOT in fused mode => fused mode implies single line of source code)
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
            nodes.push(Node::new(node_type, grammar, token.line));
        }
    }
    // Return or provide a list of errors
    (nodes, error_list)
}
