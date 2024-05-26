//! Grammars define the permissible syntax of a sequence of tokens
//!
//! Grammars are essentially large state machines and look "kind of like" a regex. The difference between the grammars here and the regex is conditional behavior.
//!
//! We have a small number of grammars (7) because the language has a strict structure -- each line must belong to one these classes of behavior. The list of grammars corresponds to the node types.

use crate::compiler_errors::{CompilerProblem, ProblemClass};
use crate::lex::{Symbol, Token};
use crate::parse::{DataType, Variable};
use crate::properties;

pub enum Grammar {
    GrammarFunctionDeclaration,
    GrammarProperties,
    GrammarVariableAssignments,
}

// -------------------- Grammar: Functions --------------------

enum GFDStages {
    Initialized,
    NameProcessed,
    SeekingArguments,
    SeekingBracket,
}

/// The grammar for declaring a function -> a big state machine
///
/// #### Stages
///
///     0: Initialized
///     1: Name processed, seeking :: or {
///     2: :: processed, seeking arguments
///     3: arguments complete, seeking {
pub struct GrammarFunctionDeclaration {
    is_valid: bool,
    done: bool,
    stage: GFDStages,
    last_symbol: Symbol,
    fn_name: String,
    arguments: Vec<Variable>,
    return_type: DataType,
}

impl GrammarFunctionDeclaration {
    pub fn new() -> GrammarFunctionDeclaration {
        GrammarFunctionDeclaration {
            is_valid: true,
            done: false,
            stage: GFDStages::Initialized,
            last_symbol: Symbol::FunctionDeclare,
            fn_name: "undefined".to_string(),
            arguments: Vec::<Variable>::new(),
            return_type: DataType::Void,
        }
    }

    /// Steps forward through a state machine, returning optional error message
    pub fn step(&mut self, next: &Token) -> Option<CompilerProblem> {
        let mut error_message: Option<CompilerProblem> = None;
        match self.stage {
            // Initial stage -> next symbol should be the fn name
            GFDStages::Initialized => match next.symbol {
                Symbol::Value => {
                    if next.text.is_ascii() {
                        self.fn_name = next.text.to_string();
                        self.stage = GFDStages::NameProcessed;
                    } else {
                        error_message = Some(CompilerProblem::new(
                            ProblemClass::Error,
                            "function name is not valid ASCII",
                            next.line,
                            next.word,
                        ));
                    }
                }
                _ => {
                    self.is_valid = false;
                    self.done = true;
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Error,
                        "function name is missing",
                        next.line,
                        next.word,
                    ));
                }
            },
            // Function has been named. Now need either a left brace (no args) or a :: (args)
            GFDStages::NameProcessed => match next.symbol {
                Symbol::BraceOpen => {
                    self.done = true;
                }
                Symbol::DoubleColon => {
                    self.stage = GFDStages::SeekingArguments;
                }
                _ => {
                    self.is_valid = false;
                    self.done = true;
                    error_message = Some(CompilerProblem::new(ProblemClass::Error, &format!("expected a '::' (if it has args) or a '{{' (if it doesn't have args) after the function name, but received '{}'.", next.text), next.line, next.word));
                }
            },
            // Function has one or more arguments.
            GFDStages::SeekingArguments => {
                if self.last_symbol == Symbol::DoubleColon || self.last_symbol == Symbol::RightArrow
                {
                    match next.symbol {
                        // If we receive a type after :: or ->, it implies that is the return type and there are no arguments
                        Symbol::TypeBool => {
                            self.stage = GFDStages::SeekingBracket;
                            self.return_type = DataType::Bool;
                        }
                        Symbol::TypeInt => {
                            self.stage = GFDStages::SeekingBracket;
                            self.return_type = DataType::Int;
                        }
                        Symbol::TypeStr => {
                            self.stage = GFDStages::SeekingBracket;
                            self.return_type = DataType::Str;
                        }
                        Symbol::TypeVoid => {
                            self.stage = GFDStages::SeekingBracket;
                            self.return_type = DataType::Void;
                        }
                        // A value here implies the argument name
                        Symbol::Value => {
                            self.arguments.push(Variable {
                                name: next.text.to_string(),
                                data_type: DataType::Void,
                                value: None,
                            });
                        }
                        _ => {
                            self.is_valid = false;
                            self.done = true;
                            error_message = Some(CompilerProblem::new(ProblemClass::Error, &format!("expected an argument name or a return type, but received '{}'. Check your function arguments.", next.text), next.line, next.word));
                        }
                    }
                } else if self.last_symbol == Symbol::Value {
                    match next.symbol {
                        // A value here would be an argument name, so we need an argument type
                        Symbol::TypeBool => {
                            self.arguments
                                .last_mut()
                                .expect("expected argument to exist")
                                .data_type = DataType::Bool;
                        }
                        Symbol::TypeInt => {
                            self.arguments
                                .last_mut()
                                .expect("expected argument to exist")
                                .data_type = DataType::Int;
                        }
                        Symbol::TypeStr => {
                            self.arguments
                                .last_mut()
                                .expect("expected argument to exist")
                                .data_type = DataType::Str;
                        }
                        Symbol::TypeVoid => {
                            self.is_valid = false;
                            self.done = true;
                            error_message = Some(CompilerProblem::new(
                                ProblemClass::Error,
                                &format!(
                                    "argument type for '{}' cannot be 'void'.",
                                    self.arguments
                                        .last()
                                        .expect("expected argument to exist")
                                        .name
                                ),
                                next.line,
                                next.word,
                            ));
                        }
                        _ => {
                            self.is_valid = false;
                            self.done = true;
                            error_message = Some(CompilerProblem::new(
                                ProblemClass::Error,
                                &format!(
                                    "need a type for argument '{}'.",
                                    self.arguments
                                        .last()
                                        .expect("expected argument to exist")
                                        .name
                                ),
                                next.line,
                                next.word,
                            ));
                        }
                    }
                } else if self.last_symbol == Symbol::TypeBool
                    || self.last_symbol == Symbol::TypeInt
                    || self.last_symbol == Symbol::TypeStr
                {
                    // We just received an argument type, so we need an arrow
                    if next.symbol != Symbol::RightArrow {
                        self.is_valid = false;
                        self.done = true;
                        error_message = Some(CompilerProblem::new(
                            ProblemClass::Error,
                            &format!(
                                "need a '->' after argument '{}'.",
                                self.arguments
                                    .last()
                                    .expect("expected argument to exist")
                                    .name
                            ),
                            next.line,
                            next.word,
                        ));
                    }
                }
            }
            GFDStages::SeekingBracket => match next.symbol {
                Symbol::BraceOpen => {
                    self.done = true;
                }
                _ => {
                    self.is_valid = false;
                    self.done = true;
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Error,
                        &format!(
                            "expected '{{', but received '{}'. Check your function arguments.",
                            next.text
                        ),
                        next.line,
                        next.word,
                    ));
                }
            },
        }
        // Update symbol register
        self.last_symbol = next.symbol;
        error_message
    }
}

// -------------------- Grammar: Properties --------------------

enum GPStages {
    Initialized,
    ExpectValues,
}

/// The Grammar for declaring a function's properties
struct GrammarProperties {
    is_valid: bool,
    done: bool,
    stage: GPStages,
    last_symbol: Symbol,
    p_list: Vec<properties::Properties>,
}

impl GrammarProperties {
    fn new() -> GrammarProperties {
        GrammarProperties {
            is_valid: true,
            done: false,
            stage: GPStages::Initialized,
            last_symbol: Symbol::PropertyDeclaration,
            p_list: Vec::<properties::Properties>::new(),
        }
    }

    /// Iterate through the line
    ///
    /// Stages:
    ///
    /// 0. Begin, expect semi-colon
    /// 1. Has double colon, expect values or new line
    fn step(&mut self, next: Token) -> Option<String> {
        let mut error_message: Option<String> = None;
        match self.stage {
            GPStages::Initialized => match next.symbol {
                Symbol::DoubleColon => {
                    self.stage = GPStages::ExpectValues;
                }
                _ => {
                    self.is_valid = false;
                    self.done = true;
                    error_message = Some(format!("property list declared on line {} is invalid. Should be `#Properties :: A B C`.", next.line));
                }
            },
            GPStages::ExpectValues => match next.symbol {
                Symbol::Value => match next.text.as_str() {
                    "Pure" => self.p_list.push(properties::Properties::Pure),
                    "Export" => self.p_list.push(properties::Properties::Export),
                    _ => {
                        self.is_valid = false;
                        self.done = true;
                        error_message = Some(format!("property list declared on line {} is invalid. Unrecognized property {}. Valid properties are:\n{:?}", next.line, next.text, properties::PROPERTY_LIST));
                    }
                },
                Symbol::Newline => {
                    if self.p_list.is_empty() {
                        println!("Warning: empty property list. A property list was declared on line {}, but no properties were provided.", next.line);
                    }
                    self.done = true;
                }
                _ => {
                    self.is_valid = false;
                    self.done = true;
                    error_message = Some(format!("Property list declared on line {} is invalid. Expected a valid property name or a new line, but received an unexpected token instead. The offending token is {}, which has symbol {:?}.", next.line, next.text, next.symbol));
                }
            },
        }
        error_message
    }
}

// -------------------- Grammar: Variable Assignment --------------------

enum AssignmentTypes {
    Const,      // const variable
    Initialize, // let x = ...
    Mutate,     // set x = ...
}

enum VariableAssignmentStages {
    DeclaringType,
    FindingName,
    HandlingValues,
}

pub struct GrammarVariableAssignments {
    is_valid: bool,
    done: bool,
    stage: VariableAssignmentStages,
    last_symbol: Symbol,
    assignment_type: AssignmentTypes,
    data_type: DataType,
    name: String,
    arguments: Vec<Token>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lex;

    #[test]
    fn declare_fn_simple_1() {
        let mut gfd = GrammarFunctionDeclaration::new();
        let line: &str = "fn add :: a int -> b int -> int {\n";
        let tokens = lex(line);
        // Skip the first token (the `fn` token)
        for t in tokens.into_iter().skip(1) {
            gfd.step(&t);
        }
        assert!(gfd.done);
        assert!(gfd.is_valid);
        assert_eq!(gfd.fn_name, "add");
        assert_eq!(gfd.arguments.len(), 2);
        // Arg 1
        assert_eq!(gfd.arguments[0].name, "a");
        assert_eq!(gfd.arguments[0].data_type, DataType::Int);
        assert!(gfd.arguments[0].value.is_none());
        // Arg 2
        assert_eq!(gfd.arguments[1].name, "b");
        assert_eq!(gfd.arguments[1].data_type, DataType::Int);
        assert!(gfd.arguments[1].value.is_none());
        assert_eq!(gfd.return_type, DataType::Int);
    }

    #[test]
    fn declare_fn_simple_2() {
        let mut gfd = GrammarFunctionDeclaration::new();
        let line: &str = "fn copy_to :: old_filepath str -> new_filepath str -> void {
        ";
        let tokens = lex(line);
        // Skip the first token (the `fn` token)
        for t in tokens.into_iter().skip(1) {
            let msg = gfd.step(&t);
            println!("{:?}", msg);
        }
        assert!(gfd.done);
        assert!(gfd.is_valid);
        assert_eq!(gfd.fn_name, "copy_to");
        assert_eq!(gfd.arguments.len(), 2);
        // Arg 1
        assert_eq!(gfd.arguments[0].name, "old_filepath");
        assert_eq!(gfd.arguments[0].data_type, DataType::Int);
        assert!(gfd.arguments[0].value.is_none());
        // Arg 2
        assert_eq!(gfd.arguments[1].name, "new_filepath");
        assert_eq!(gfd.arguments[1].data_type, DataType::Str);
        assert!(gfd.arguments[1].value.is_none());
        assert_eq!(gfd.return_type, DataType::Void);
    }
}
