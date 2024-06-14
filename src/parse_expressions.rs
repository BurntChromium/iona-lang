//! # Expression Parsing
//!
//! This is a (large) submodule of the parser dedicated to parsing expressions (such as `sqrt 37`).
//!
//! For now we only support prefix operators (like lisp)
//!
//! It will probably eventually be an implementation of a Pratt Parser (or a Top Down Operator Precedence Parser).
//!
//! All named functions are prefix operations. Some basic mathematical operations (and potentially overloads?) are infix operations.

use std::collections::BTreeMap;

use crate::compiler_errors::{CompilerProblem, ProblemClass};
use crate::lex::{Symbol, Token};
use crate::parse::FunctionData;

pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Inverse,
    Function { name: String },
}

impl Operator {
    fn binding_power(&self) -> u8 {
        match self {
            Self::Add => 20,
            Self::Subtract => 20,
            Self::Multiply => 30,
            Self::Divide => 30,
            Self::Negate => 40,
            Self::Inverse => 40,
            _ => 10,
        }
    }

    fn from_symbol(symbol: Symbol) -> Option<Operator> {
        match symbol {
            Symbol::OpPlus => Some(Operator::Add),
            Symbol::OpMinus => Some(Operator::Subtract),
            Symbol::OpMul => Some(Operator::Multiply),
            Symbol::OpDiv => Some(Operator::Divide),
            _ => None,
        }
    }
}

pub enum Expression {
    Prefix { op: Operator, args: Vec<Object> },
    // Infix {
    //     left: Box<Object>,
    //     op: Operator,
    //     right: Option<Box<Object>>,
    // },
}

impl Expression {
    pub fn get_bp(&self) -> u8 {
        match &self {
            Expression::Prefix { op, .. } => op.binding_power(),
            // Expression::Infix { op, .. } => op.binding_power()
        }
    }
}

pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Symbol(String),
}

impl Literal {
    /// Convert a string (from a token) into a literal value
    pub fn from_str(text: &str) -> Result<Literal, CompilerProblem> {
        // Handle booleans
        if text == "true" {
            return Ok(Literal::Bool(true));
        } else if text == "false" {
            return Ok(Literal::Bool(false));
        // Handle strings
        } else if text.starts_with("\"") {
            if text.ends_with("\"") {
                return Ok(Literal::Str(text.to_string()));
            } else {
                return Err(
                    CompilerProblem::new(ProblemClass::Error, "a string literal had an unclosed parenthesis", "if this isn't a string, remove the opening parenthesis, otherwise, close the parenthesis", 0, 0).into()
                );
            }
        } else if text.ends_with("\"") {
            return Err(
                CompilerProblem::new(ProblemClass::Error, "a string literal had an unopened parenthesis", "if this isn't a string, remove the closing parenthesis, otherwise, close the parenthesis", 0, 0).into()
            );
        }
        // Handle integers
        let maybe_int = text.parse::<i64>();
        if maybe_int.is_ok() {
            return Ok(Literal::Int(maybe_int.unwrap()));
        }
        // Handle floating point numbers
        let maybe_float = text.parse::<f64>();
        if maybe_float.is_ok() {
            return Ok(Literal::Float(maybe_float.unwrap()));
        }
        // Return an error for everything else
        Err(CompilerProblem::new(
            ProblemClass::Error,
            "unable to parse literal value",
            "check for syntax errors",
            0,
            0,
        )
        .into())
    }
}

pub enum Object {
    Operation(Expression),
    Value(Literal),
}

pub fn push_fn_to_stack(
    token: &Token,
    op: Operator,
    arg_count: usize,
    stack: &mut Vec<Object>,
) -> Option<CompilerProblem> {
    let mut args: Vec<Object> = Vec::with_capacity(arg_count);
    // Do we have enough objects on the stack to satisfy the fn call?
    if stack.len() < arg_count {
        return Some(CompilerProblem::new(
            ProblemClass::Error,
            &format!("not enough arguments when calling function {}", &token.text),
            "partial functions are not yet supported by the compiler",
            token.line,
            token.word,
        ));
    }
    // Pop the last N objects off the stack and move them into the function's arguments (N == fn.args.len)
    args.extend(stack.drain(stack.len() - arg_count..));
    // Finally, push this fn onto the stack
    stack.push(Object::Operation(Expression::Prefix { op: op, args: args }));
    None
}

/// Currently only supports prefix operations
pub fn parse_expression(
    tokens: &Vec<Token>,
    fn_table: &BTreeMap<String, FunctionData>,
) -> Result<Object, CompilerProblem> {
    // Sanity check
    if tokens.is_empty() {
        return Err(CompilerProblem::new(
            ProblemClass::Error,
            "expression has no tokens",
            "make sure to provide a value or call a function here",
            0,
            0,
        ));
    }
    // We will push and pop objects/expressions onto a stack
    let mut stack: Vec<Object> = Vec::with_capacity(tokens.len());
    // Iterate backwards over the tokens
    for token in tokens.iter().rev() {
        match token.symbol {
            Symbol::OpPlus => {
                let outcome = push_fn_to_stack(token, Operator::Add, 2, &mut stack);
                if let Some(e) = outcome {
                    return Err(e);
                }
            }
            Symbol::OpMinus => {
                let outcome = push_fn_to_stack(token, Operator::Subtract, 2, &mut stack);
                if let Some(e) = outcome {
                    return Err(e);
                }
            }
            Symbol::OpMul => {
                let outcome = push_fn_to_stack(token, Operator::Multiply, 2, &mut stack);
                if let Some(e) = outcome {
                    return Err(e);
                }
            }
            Symbol::OpDiv => {
                let outcome = push_fn_to_stack(token, Operator::Divide, 2, &mut stack);
                if let Some(e) = outcome {
                    return Err(e);
                }
            }
            Symbol::Value => {
                // Check if it's a function
                if fn_table.contains_key(&token.text) {
                    // Check how many arguments it takes
                    let arg_count = fn_table.get(&token.text).unwrap().args.len();
                    let outcome = push_fn_to_stack(
                        token,
                        Operator::Function {
                            name: token.text.clone(),
                        },
                        arg_count,
                        &mut stack,
                    );
                    if let Some(e) = outcome {
                        return Err(e);
                    }
                } else {
                    // If not, it must be a value
                    match Literal::from_str(&token.text) {
                        Ok(lit) => {
                            stack.push(Object::Value(lit));
                        }
                        // If it's not a value, throw an error
                        Err(mut e) => {
                            e.line = token.line;
                            e.word_index = token.word;
                            return Err(e);
                        }
                    }
                }
            }
            _ => {
                return Err(CompilerProblem::new(
                    ProblemClass::Error,
                    "unimplemented symbol found in expression",
                    "please wait for compiler update",
                    token.line,
                    token.word,
                ))
            }
        }
    }
    if stack.is_empty() {
        let line_no: usize;
        let word: usize;
        if let Some(t) = tokens.get(0) {
            line_no = t.line;
            word = t.word;
        } else {
            line_no = 0;
            word = 0;
        }
        return Err(CompilerProblem::new(
            ProblemClass::Error,
            "empty expression",
            "make sure to provide a value or call a function here",
            line_no,
            word,
        ));
    } else if stack.len() == 1 {
        return Ok(stack.pop().unwrap());
    } else {
        return Err(CompilerProblem::new(
            ProblemClass::Error,
            "too many objects left on the expression stack after parsing",
            "you probably have passed too many arguments to a function",
            tokens.last().unwrap().line,
            tokens.last().unwrap().word,
        ));
    }
}
