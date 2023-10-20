//! Parse constructs an abstract syntax tree or equivalent
//! 
//! Organizational note: the syntax of permissible tokens is defined by the `grammar` crate.

/// Object represents something that has been parsed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeType {
    FunctionDeclaration,
    PropertyDeclaration,
    ContractDeclaration,
    VariableDeclaration,
    FunctionInvocation,
    TypeDeclaration,
    ValueLiteral,
}

pub enum DataType {
    Void,
    Int,
    Str,
    Bool,
}

pub struct Node {
    node_type: NodeType,
    source_line: usize,
    source_string: String,
    children: Vec<Node>
}

pub struct Variable {
    pub name: String,
    pub data_type: DataType
}

// pub fn parse(tokens: Token) -> Node {

// }