//! Parse constructs an abstract syntax tree or equivalent
//!
//! Organizational note: the syntax of permissible tokens is defined by the `grammar` crate.

use std::fmt::Debug;

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

// pub fn parse(tokens: Token) -> Node {

// }
