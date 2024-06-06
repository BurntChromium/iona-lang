//! The `parse` module constructs an abstract syntax tree (AST) or an equivalent
//!
//! Organizational note: the arrangements of permissible token sequences are defined by the `grammar` crate. Each line of Iona code corresponds to one singular Grammar, and can be parsed into that grammar independently.
//!
//! We represent our AST as a flat list of `Nodes`, and each `Node` is assigned a Grammar and some metadata.

use std::collections::BTreeMap;
use std::fmt::Debug;

use crate::compiler_errors::{CompilerProblem, ProblemClass};
use crate::grammars::Grammar;
use crate::lex::{Symbol, Token, VALID_EXPRESSION_TOKENS};
use crate::permissions::Permissions;
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
    Comment,                     // done
    FunctionDeclaration,         // done
    PropertyDeclaration,         // done
    PermissionsDeclaration,      // done
    ContractDeclaration,         // TODO
    VariableAssignment,          // done
    TypeDeclaration,             // newtype, TODO
    Expression,                  // TODO
    EffectualFunctionInvocation, // TODO
    ImportStatement,             // done
    ReturnStatement,             // done
    CloseScope,                  // done
    Empty,                       // done
}

/// Primitive data types (i.e. types not held in a container or struct)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PrimitiveDataType {
    Void,
    Int,
    Float,
    Str,
    Bool,
}

impl PrimitiveDataType {
    pub fn from_symbol(sym: Symbol) -> Option<PrimitiveDataType> {
        match sym {
            Symbol::TypeVoid => Some(PrimitiveDataType::Void),
            Symbol::TypeInt => Some(PrimitiveDataType::Int),
            Symbol::TypeFloat => Some(PrimitiveDataType::Float),
            Symbol::TypeStr => Some(PrimitiveDataType::Str),
            Symbol::TypeBool => Some(PrimitiveDataType::Bool),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            PrimitiveDataType::Void => "void",
            PrimitiveDataType::Bool => "bool",
            PrimitiveDataType::Int => "int",
            PrimitiveDataType::Float => "float",
            PrimitiveDataType::Str => "char",
        }
    }
}

pub trait Data: Debug {
    fn box_clone(&self) -> Box<dyn Data>;
}

impl Clone for Box<dyn Data> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[derive(Debug)]
/// A Node in the AST (represented as a list)
///
/// `parent_node_line` is principally used to track scope
pub struct Node {
    pub node_type: NodeType,
    pub grammar: Grammar,
    pub source_line: usize,
    pub parent_node_line: Option<usize>,
}

impl Node {
    pub fn new(node_type: NodeType, grammar: Grammar, source_line: usize) -> Node {
        Node {
            node_type,
            grammar,
            source_line,
            parent_node_line: None,
        }
    }
}

#[derive(Debug, Clone)]
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
        let mut grammar: Grammar = match token.symbol {
            // Handle imports
            Symbol::Import => {
                node_type = NodeType::ImportStatement;
                Grammar::new(token.symbol)
            }
            // Handle function declaration
            Symbol::FunctionDeclare => {
                node_type = NodeType::FunctionDeclaration;
                Grammar::new(token.symbol)
            }
            // Handle property declarations (pass in dummy value to signal type)
            Symbol::PropertyDeclaration => {
                node_type = NodeType::PropertyDeclaration;
                Grammar::new(token.symbol)
            }
            // Handle permissions declarations (pass in dummy value to signal type)
            Symbol::PermissionsDeclaration => {
                node_type = NodeType::PermissionsDeclaration;
                Grammar::new(token.symbol)
            }
            // Handle variable declarations
            Symbol::Set | Symbol::Let => {
                node_type = NodeType::VariableAssignment;
                Grammar::new(token.symbol)
            }
            // Handle contracts
            Symbol::ContractPre | Symbol::ContractPost | Symbol::ContractInvariant => {
                node_type = NodeType::ContractDeclaration;
                Grammar::new(token.symbol)
            }
            // Handle return statements
            Symbol::Return => {
                node_type = NodeType::ReturnStatement;
                Grammar::new(token.symbol)
            }
            // Handle scope closes
            Symbol::BraceClose => {
                node_type = NodeType::CloseScope;
                Grammar::new(token.symbol)
            }
            // Skip comments with empty grammar
            Symbol::Comment => {
                node_type = NodeType::Comment;
                Grammar::new(token.symbol)
            }
            // Skip newlines
            Symbol::Newline => {
                continue;
            }
            _ => {
                if VALID_EXPRESSION_TOKENS.contains(&token.symbol) {
                    node_type = NodeType::Expression;
                    Grammar::new(token.symbol)
                } else {
                    node_type = NodeType::Empty;
                    Grammar::new(token.symbol)
                }
            }
        };
        // We will get 1 "error" per token (error can be None!)
        let mut errors: Vec<Option<CompilerProblem>> = Vec::new();
        let future = iterator.clone().peekable();
        for t in future {
            // Loop until the grammar finishes
            if !grammar.is_done() {
                errors.push(grammar.step(t));
            } else {
                break;
            }
        }
        // Then force the iterator to catch up
        if errors.len() > 1 {
            iterator.nth(errors.len().saturating_sub(1));
        }
        // Check for errors (this happens after skip because consumes iterator)
        let mut okay = true;
        for e in errors {
            if let Some(problem) = e {
                if problem.class == ProblemClass::Error {
                    okay = false;
                }
                error_list.push(problem);
            }
        }
        if okay {
            nodes.push(Node::new(node_type, grammar, token.line));
        }
    }
    // Return or provide a list of errors
    (nodes, error_list)
}

/// Data contained within the function table for easy type checking
#[derive(Debug)]
pub struct FunctionData {
    pub args: Vec<Variable>,
    pub return_type: PrimitiveDataType,
    pub properties: Vec<Properties>,
    pub permissions: Vec<Permissions>,
}

impl FunctionData {
    pub fn new() -> FunctionData {
        FunctionData {
            args: Vec::new(),
            return_type: PrimitiveDataType::Void,
            properties: Vec::new(),
            permissions: Vec::new(),
        }
    }

    pub fn arity(&self) -> usize {
        self.args.len()
    }
}

// -------------------- AST Post Processing --------------------

/// Get the scopes of various objects in the AST
pub fn compute_scopes(nodes: &mut Vec<Node>) -> Vec<CompilerProblem> {
    let mut scope_depth: usize = 0;
    let mut last_seen_scope_line: usize = 0;
    let mut errors: Vec<CompilerProblem> = Vec::new();
    for node in nodes {
        match node.node_type {
            NodeType::FunctionDeclaration => {
                if scope_depth > 0 {
                    errors.push(CompilerProblem::new(ProblemClass::Error, "issue with function declaration: either there's an unclosed scope or you tried to declare one function inside another", "check for missing braces `}`, and don't try to declare a nested function", node.source_line, 0));
                } else {
                    last_seen_scope_line = node.source_line;
                    scope_depth += 1;
                }
            }
            NodeType::CloseScope => {
                node.parent_node_line = Some(last_seen_scope_line);
                scope_depth -= 1;
            }
            // TODO: handle match statements
            _ => {
                node.parent_node_line = Some(last_seen_scope_line);
            }
        }
    }
    errors
}

/// Construct a function table from the nodes we get from parse
pub fn populate_function_table(
    nodes: &Vec<Node>,
) -> Result<BTreeMap<String, FunctionData>, Vec<CompilerProblem>> {
    let mut table: BTreeMap<String, FunctionData> = BTreeMap::new();
    let mut errors: Vec<CompilerProblem> = Vec::new();
    let mut data: Option<FunctionData> = None;
    let mut function_name: Option<String> = None;
    let mut function_line: usize = 0;
    for node in nodes {
        if node.node_type == NodeType::FunctionDeclaration {
            data = Some(FunctionData::new());
            function_line = node.source_line;
            match &node.grammar {
                Grammar::Function(fg) => {
                    data.as_mut().unwrap().args = fg.arguments.clone();
                    data.as_mut().unwrap().return_type = fg.return_type;
                    function_name = Some(fg.fn_name.clone());
                }
                _ => {}
            }
        } else {
            // We can assume every property is declared after a fn unless there's a syntax error
            match &node.grammar {
                Grammar::Property(pg) => match data {
                    Some(ref mut d) => d.properties = pg.p_list.clone(),
                    None => {
                        errors.push(CompilerProblem::new(
                            ProblemClass::Error,
                            "property list declared outside of function",
                            "make sure all properties are inside a function",
                            node.source_line,
                            0,
                        ));
                    }
                },
                Grammar::Permission(pg) => match data {
                    Some(ref mut d) => d.permissions = pg.p_list.clone(),
                    None => {
                        errors.push(CompilerProblem::new(
                            ProblemClass::Error,
                            "property list declared outside of function",
                            "make sure all properties are inside a function",
                            node.source_line,
                            0,
                        ));
                    }
                },
                _ => {}
            }
            // If we see a scope closure corresponding to our function, then package up our data
            if node.node_type == NodeType::CloseScope
                && node.parent_node_line == Some(function_line)
            {
                if data.is_some() {
                    table.insert(function_name.clone().unwrap(), data.unwrap());
                }
                data = None;
                function_name = None;
            }
        }
    }
    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(table)
    }
}

// -------------------- Unit Tests --------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lex;

    #[test]
    fn parse_line_1() {
        let code: &str = "fn five :: int {";
        let tokens = lex(code);
        let (nodes, errors) = parse(tokens);
        assert_eq!(nodes.len(), 1);
        assert!(errors.is_empty());
        assert_eq!(nodes[0].node_type, NodeType::FunctionDeclaration);
    }

    #[test]
    fn parse_function_1() {
        let code: &str = "// Empty comment
        fn five :: int {
            return 5
        }";
        let tokens = lex(code);
        let (nodes, errors) = parse(tokens);
        assert_eq!(nodes.len(), 5);
        assert!(errors.is_empty());
        assert_eq!(nodes[0].node_type, NodeType::Comment);
        assert_eq!(nodes[1].node_type, NodeType::FunctionDeclaration);
        assert_eq!(nodes[2].node_type, NodeType::ReturnStatement);
        assert_eq!(nodes[3].node_type, NodeType::Expression);
        assert_eq!(nodes[4].node_type, NodeType::CloseScope);
        match &nodes[1].grammar {
            Grammar::Function(g) => assert_eq!(g.fn_name, "five"),
            _ => {}
        }
    }

    #[test]
    fn parse_function_2() {
        let code: &str = "// This function adds two numbers
        fn add :: a int -> b int -> int {
            #Properties :: Pure Export
            return a + b
        }";
        let tokens = lex(code);
        let (nodes, errors) = parse(tokens);
        assert_eq!(nodes.len(), 6);
        assert!(errors.is_empty());
        assert_eq!(nodes[0].node_type, NodeType::Comment);
        assert_eq!(nodes[1].node_type, NodeType::FunctionDeclaration);
        assert_eq!(nodes[2].node_type, NodeType::PropertyDeclaration);
        assert_eq!(nodes[3].node_type, NodeType::ReturnStatement);
        assert_eq!(nodes[4].node_type, NodeType::Expression);
        assert_eq!(nodes[5].node_type, NodeType::CloseScope);
    }

    #[test]
    fn populate_function_table_1() {
        let code: &str = "// This function adds two numbers
        fn add :: a int -> b int -> int {
            #Properties :: Pure Export
            return a + b
        }";
        let tokens = lex(code);
        let (mut nodes, _) = parse(tokens);
        compute_scopes(&mut nodes);
        let f_table = populate_function_table(&nodes);
        assert!(f_table.is_ok());
        let function_table = f_table.unwrap();
        for (name, data) in function_table.iter() {
            println!("{name}: {:#?}", data);
        }
        assert!(function_table.get("add").is_some());
        assert_eq!(
            function_table.get("add").unwrap().return_type,
            PrimitiveDataType::Int
        );
    }
}
