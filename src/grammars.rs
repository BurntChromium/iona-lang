//! Grammars define the permissible syntax of a sequence of tokens. Each line has one (and only one) possible grammar.
//!
//! Grammars are essentially large state machines and look "kind of like" a regex. The difference between the grammars here and the regex is conditional behavior.
//!
//! We have a small number of grammars (7) because the language has a strict structure -- each line must belong to one these classes of behavior. The list of grammars corresponds to the node types.

use std::fmt::Debug;

use crate::compiler_errors::{CompilerProblem, ProblemClass};
use crate::lex::{Symbol, Token, BANNED_RHS_SYMBOLS};
use crate::parse::{DataType, Variable};
use crate::properties;

pub trait Grammar: Debug {
    fn step(&mut self, next: &Token) -> Option<CompilerProblem>;
}

// -------------------- Grammar: Functions --------------------

#[derive(Debug)]
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
#[derive(Debug)]
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
}

impl Grammar for GrammarFunctionDeclaration {
    /// Steps forward through a state machine, returning optional error message
    fn step(&mut self, next: &Token) -> Option<CompilerProblem> {
        if self.done {
            return None;
        }
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
                            "choose a different function name",
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
                        "choose a name for this function",
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
                    error_message = Some(CompilerProblem::new(ProblemClass::Error, &format!("expected a '::' (if it has args) or a '{{' (if it doesn't have args) after the function name, but received '{}'.", next.text), "functions should look like this: `fn foo :: a int -> int`", next.line, next.word));
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
                            error_message = Some(CompilerProblem::new(ProblemClass::Error, &format!("expected an argument name or a return type, but received '{}'.", next.text), "check your function arguments.", next.line, next.word));
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
                                "the `void` keyword is only valid as a return type",
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
                                    "argument '{}' has no type information.",
                                    self.arguments
                                        .last()
                                        .expect("expected argument to exist")
                                        .name
                                ),
                                "add a type for this argument",
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
                                "missing a '->' after argument '{}'.",
                                self.arguments
                                    .last()
                                    .expect("expected argument to exist")
                                    .name
                            ),
                            "add a `->` to separate two arguments",
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
                        &format!("expected '{{', but received '{}'.", next.text),
                        "check your function arguments.",
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

#[derive(Debug)]
enum GPStages {
    Initialized,
    ExpectValues,
}

/// The Grammar for declaring a function's properties
#[derive(Debug)]
pub struct GrammarProperties {
    is_valid: bool,
    done: bool,
    stage: GPStages,
    last_symbol: Symbol,
    p_list: Vec<properties::Properties>,
}

impl GrammarProperties {
    pub fn new() -> GrammarProperties {
        GrammarProperties {
            is_valid: true,
            done: false,
            stage: GPStages::Initialized,
            last_symbol: Symbol::PropertyDeclaration,
            p_list: Vec::<properties::Properties>::new(),
        }
    }
}

impl Grammar for GrammarProperties {
    /// Iterate through the line
    ///
    /// Stages:
    ///
    /// 0. Begin, expect semi-colon
    /// 1. Has double colon, expect values or new line
    fn step(&mut self, next: &Token) -> Option<CompilerProblem> {
        let mut error_message: Option<CompilerProblem> = None;
        match self.stage {
            GPStages::Initialized => match next.symbol {
                Symbol::DoubleColon => {
                    self.stage = GPStages::ExpectValues;
                }
                _ => {
                    self.is_valid = false;
                    self.done = true;
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Error,
                        &format!(
                            "property list is invalid - expected a `::` but found {}",
                            next.text
                        ),
                        "a property list should look like this: `#Properties :: A B C`.",
                        next.line,
                        next.word,
                    ));
                }
            },
            GPStages::ExpectValues => match next.symbol {
                Symbol::Value => match next.text.as_str() {
                    "Pure" => self.p_list.push(properties::Properties::Pure),
                    "Export" => self.p_list.push(properties::Properties::Export),
                    _ => {
                        self.is_valid = false;
                        self.done = true;
                        error_message = Some(CompilerProblem::new(
                            ProblemClass::Error,
                            &format!("unrecognized property {}.", next.text),
                            &format!("valid properties are:\n{:?}", properties::PROPERTY_LIST),
                            next.line,
                            next.word,
                        ));
                    }
                },
                Symbol::Newline => {
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Warning,
                        "empty property list",
                        "either remove the property list or add valid properties",
                        next.line,
                        next.word,
                    ));
                    self.done = true;
                }
                _ => {
                    self.is_valid = false;
                    self.done = true;
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Error,
                        &format!("expected a valid property name or a new line, but received an unexpected token instead. the offending token is {}, which has symbol {:?}.", next.text, next.symbol),
                        "a property list should look like this: `#Properties :: A B C`.",
                        next.line,
                        next.word,
                    ));
                }
            },
        }
        error_message
    }
}

// -------------------- Grammar: Variable Assignment --------------------

#[derive(Debug)]
enum AssignmentTypes {
    Initialize, // let x = ...
    Mutate,     // set x = ...
}

#[derive(Debug)]
enum VariableAssignmentStages {
    FindingName,
    CheckingForIndex,
    GettingIndexValue,
    DeclaringType,
    SeekingTypeName,
    HandlingValues,
}

#[derive(Debug)]
pub struct GrammarVariableAssignments {
    is_valid: bool,
    done: bool,
    stage: VariableAssignmentStages,
    last_symbol: Symbol,
    assignment_type: AssignmentTypes,
    type_provided: bool, // if provided, type check (later), otherwise run type inference (later)
    data_type: DataType,
    name: String,
    mutable: bool,
    index_text: Option<String>,
    arguments: Vec<Token>,
}

impl GrammarVariableAssignments {
    pub fn new() -> GrammarVariableAssignments {
        GrammarVariableAssignments {
            is_valid: true,
            done: false,
            stage: VariableAssignmentStages::DeclaringType,
            last_symbol: Symbol::Let, // May not be strictly true, but not relevant
            assignment_type: AssignmentTypes::Initialize,
            type_provided: false,
            data_type: DataType::Void,
            name: "unknown".to_string(),
            mutable: false,
            index_text: None,
            arguments: Vec::<Token>::new(),
        }
    }
}

impl Grammar for GrammarVariableAssignments {
    fn step(&mut self, next: &Token) -> Option<CompilerProblem> {
        let mut error_message: Option<CompilerProblem> = None;
        match self.stage {
            VariableAssignmentStages::FindingName => match next.symbol {
                Symbol::Value => {
                    if next.text.is_ascii() {
                        self.name = next.text.to_string();
                        self.stage = VariableAssignmentStages::DeclaringType;
                    } else {
                        CompilerProblem::new(
                            ProblemClass::Error,
                            &format!("this variable's name is not valid ASCII: {}", next.text),
                            "rename the variable",
                            next.line,
                            next.word,
                        );
                        self.is_valid = false;
                        self.done = true;
                    }
                }
                _ => {
                    error_message = Some(
                        CompilerProblem::new(ProblemClass::Error, &format!("expected a variable name, but found a system reserved keyword instead (found `{}`", next.text), "try using a different variable name", next.line, next.word)
                    );
                    self.is_valid = false;
                    self.done = true;
                }
            },
            VariableAssignmentStages::CheckingForIndex => match next.symbol {
                Symbol::At => self.stage = VariableAssignmentStages::GettingIndexValue,
                _ => self.stage = VariableAssignmentStages::DeclaringType,
            },
            VariableAssignmentStages::GettingIndexValue => match next.symbol {
                Symbol::Value => self.index_text = Some(next.text.to_string()),
                _ => {
                    error_message = Some(
                        CompilerProblem::new(ProblemClass::Error, &format!("expected an index, but found a system reserved keyword instead (found `{}`", next.text), "indices should be either a number `37` or a range `0..2`", next.line, next.word)
                    );
                    self.is_valid = false;
                    self.done = true;
                }
            },
            VariableAssignmentStages::DeclaringType => match next.symbol {
                // Double colon implies we're going to get a type
                Symbol::DoubleColon => {
                    self.type_provided = true;
                    self.stage = VariableAssignmentStages::SeekingTypeName;
                }
                // Equals sign implies no type present
                Symbol::EqualSign => self.stage = VariableAssignmentStages::HandlingValues,
                _ => {
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Error,
                        &format!(
                            "expected a `::` or a `=` after the variable name, but found `{}`",
                            next.text
                        ),
                        "declare a variable's type with `::` or give it a value of `=`",
                        next.line,
                        next.word,
                    ));
                    self.is_valid = false;
                    self.done = true;
                }
            },
            VariableAssignmentStages::SeekingTypeName => match next.symbol {
                Symbol::TypeBool => self.data_type = DataType::Bool,
                Symbol::TypeInt => self.data_type = DataType::Int,
                Symbol::TypeFloat => self.data_type = DataType::Float,
                Symbol::TypeStr => self.data_type = DataType::Str,
                Symbol::TypeVoid => self.data_type = DataType::Void,
                _ => {
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Error,
                        &format!("expected a type name, but found `{}`", next.text),
                        "provide a valid type such as `str` or `int`",
                        next.line,
                        next.word,
                    ));
                    self.is_valid = false;
                    self.done = true;
                }
            },
            VariableAssignmentStages::HandlingValues => {
                if next.symbol == Symbol::Newline {
                    if self.arguments.is_empty() {
                        error_message = Some(
                            CompilerProblem::new(ProblemClass::Error, &format!("expected an expression (a 'right hand side') for the value of {}, but received a newline instead", self.name), "provide a value for the variable", next.line, next.word)
                        );
                        self.is_valid = false;
                        self.done = true;
                    }
                }
                // These symbols are not allowed on RHS of expression
                else if BANNED_RHS_SYMBOLS.contains(&next.symbol) {
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Error,
                        &format!(
                            "received an illegal keyword in the assignment of {}: {}",
                            self.name, next.text
                        ),
                        "your variable assignment should be an expression",
                        next.line,
                        next.word,
                    ));
                    self.is_valid = false;
                    self.done = true;
                } else {
                    self.arguments.push(next.clone())
                }
            }
        }
        error_message
    }
}

// -------------------- Unit Tests --------------------

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
        let line: &str = "fn copy_to :: old_filepath str -> new_filepath str -> void {";
        let tokens = lex(line);
        // Skip the first token (the `fn` token)
        for t in tokens.into_iter().skip(1) {
            gfd.step(&t);
        }
        assert!(gfd.done);
        assert!(gfd.is_valid);
        assert_eq!(gfd.fn_name, "copy_to");
        assert_eq!(gfd.arguments.len(), 2);
        // Arg 1
        assert_eq!(gfd.arguments[0].name, "old_filepath");
        assert_eq!(gfd.arguments[0].data_type, DataType::Str);
        assert!(gfd.arguments[0].value.is_none());
        // Arg 2
        assert_eq!(gfd.arguments[1].name, "new_filepath");
        assert_eq!(gfd.arguments[1].data_type, DataType::Str);
        assert!(gfd.arguments[1].value.is_none());
        assert_eq!(gfd.return_type, DataType::Void);
    }

    #[test]
    fn declare_fn_no_name() {
        let mut gfd = GrammarFunctionDeclaration::new();
        let line: &str = "fn :: old_filepath str -> new_filepath str -> void {";
        let tokens = lex(line);
        let mut errors: Vec<Option<CompilerProblem>> = Vec::new();
        // Skip the first token (the `fn` token)
        for t in tokens.into_iter().skip(1) {
            errors.push(gfd.step(&t));
        }
        assert!(gfd.done);
        assert!(!gfd.is_valid);
        assert!(errors[0].is_some());
        assert_eq!(
            errors[0].as_ref().unwrap().message,
            "function name is missing"
        );
    }
}
