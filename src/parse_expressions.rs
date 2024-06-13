//! # Expression Parsing
//!
//! This is a (large) submodule of the parser dedicated to parsing expressions (such as `sqrt 37`). It is an implementation of a Pratt Parser (or a Top Down Operator Precedence Parser).
//!
//! All named functions are prefix operations. Some basic mathematical operations (and potentially overloads?) are infix operations.

use crate::compiler_errors::{CompilerProblem, ProblemClass};
use crate::lex::Token;

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
}

pub enum Expression {
    Prefix {
        op: Operator,
        args: Vec<Object>,
    },
    Infix {
        left: Box<Object>,
        op: Operator,
        right: Box<Object>,
    },
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

pub fn parse_expression(tokens: &Vec<Token>) -> Result<Object, CompilerProblem> {
    // let mut lbp: u8 = 0;
    for _token in tokens.iter() {
        // Check if it's a function

        // If not, it must be a value
        match Literal::from_str(&tokens[0].text) {
            Ok(lit) => return Ok(Object::Value(lit)),
            Err(mut e) => {
                e.line = tokens[0].line;
                e.word_index = tokens[0].word;
                return Err(e);
            }
        }
    }
    Ok(Object::Value(Literal::Bool(true)))
}
