//! Grammars define the permissible syntax of a sequence of tokens
//! 
//! Grammars are essentially large state machines and look "kind of like" a regex. The difference between the grammars here and the regex is conditional behavior.

use crate::lex::{Symbol, Token};
use crate::parse::{Variable, DataType};
use crate::properties;

// -------------------- Grammar: Functions --------------------

/// The grammar for declaring a function -> a big state machine
/// 
/// #### Stages
/// 
///     0: Initialized
/// 
///     1: Name processed, seeking :: or {
/// 
///     2: :: processed, seeking arguments
/// 
///     3: arguments complete, seeking {
struct GrammarFunctionDeclaration {
    is_valid: bool,
    done: bool,
    stage: u8,
    last_symbol: Symbol,
    fn_name: String,
    arguments: Vec<Variable>,
    return_type: DataType
}

impl GrammarFunctionDeclaration {
    fn new() -> GrammarFunctionDeclaration {
        GrammarFunctionDeclaration { 
            is_valid: true, 
            done: false,
            stage: 0,
            last_symbol: Symbol::FunctionDeclare, 
            fn_name: "undefined".to_string(), 
            arguments: Vec::<Variable>::new(),
            return_type: DataType::Void 
        }
    }

    // Steps forward through a state machine, returning optional error message
    fn step(&mut self, next: Token) -> Option<String> {
        let mut error_message: Option<String> = None;
        match self.stage {
            // Initial stage -> next symbol should be the fn name
            0 => {
                match next.symbol {
                    Symbol::Value => {
                        self.fn_name = next.text;
                        self.stage = 1;
                    },
                    _ => {
                        self.is_valid = false;
                        self.done = true;
                        error_message = Some(format!("Function declared on line {} is invalid. Missing a function name", next.line));
                    }
                }
            },
            // Function has been named. Now need either a left brace (no args) or a :: (args)
            1 => {
                match next.symbol {
                    Symbol::BraceLeft => {
                        self.done = true;
                    }
                    Symbol::DoubleColon => {
                        self.stage = 2;
                    },
                    _ => {
                        self.is_valid = false;
                        self.done = true;
                        error_message = Some(format!("Function declared on line {} is invalid. Expected a '::' (if it has args) or a '{{' (if it doesn't have args) after the function name, but received '{}'.", next.line, next.text));
                    }
                }
            },
            // Function has one or more arguments.
            2 => {
                if self.last_symbol == Symbol::DoubleColon || self.last_symbol == Symbol::RightArrow {
                    match next.symbol {
                        // If we receive a type after :: or ->, it implies that is the return type and there are no arguments
                        Symbol::TypeBool => {
                            self.stage = 3;
                            self.return_type = DataType::Bool;
                        },
                        Symbol::TypeInt => {
                            self.stage = 3;
                            self.return_type = DataType::Int;
                        },
                        Symbol::TypeStr => {
                            self.stage = 3;
                            self.return_type = DataType::Str;
                        },
                        Symbol::TypeVoid => {
                            self.stage = 3;
                            self.return_type = DataType::Void;
                        },
                        // A value here implies the argument name
                        Symbol::Value => {
                            self.arguments.push(Variable { name: next.text, data_type: DataType::Void });
                        },
                        _ => {
                            self.is_valid = false;
                            self.done = true;
                            error_message = Some(format!("Function declared on line {} is invalid. Expected an argument name or a return type, but received '{}'. Check your function arguments.", next.line, next.text));
                        }
                    }
                } else if self.last_symbol == Symbol::Value {
                    match next.symbol {
                        // A value here would be an argument name, so we need an argument type
                        Symbol::TypeBool => {
                            self.arguments.last_mut().expect("expected argument to exist").data_type = DataType::Bool;
                        },
                        Symbol::TypeInt => {
                            self.arguments.last_mut().expect("expected argument to exist").data_type = DataType::Int;
                        },
                        Symbol::TypeStr => {
                            self.arguments.last_mut().expect("expected argument to exist").data_type = DataType::Str;
                        },
                        Symbol::TypeVoid => {
                            self.is_valid = false;
                            self.done = true;
                            error_message = Some(format!("Function declared on line {} is invalid. Argument type for '{}' cannot be 'void'.", next.line, self.arguments.last().expect("expected argument to exist").name));
                        },
                        _ => {
                            self.is_valid = false;
                            self.done = true;
                            error_message = Some(format!("Function declared on line {} is invalid. Need a type for argument '{}'.", next.line, self.arguments.last().expect("expected argument to exist").name));
                        }
                    }
                } else if self.last_symbol == Symbol::TypeBool || self.last_symbol == Symbol::TypeInt || self.last_symbol == Symbol::TypeStr {
                    // We just received an argument type, so we need an arrow
                    if next.symbol != Symbol::RightArrow {
                        self.is_valid = false;
                            self.done = true;
                            error_message = Some(format!("Function declared on line {} is invalid. Need a '->' after argument '{}'.", next.line, self.arguments.last().expect("expected argument to exist").name));
                    }
                }
            },
            3 => {
                match next.symbol {
                    Symbol::BraceLeft => {
                        self.done = true;
                    },
                    _ => {
                        self.is_valid = false;
                        self.done = true;
                        error_message = Some(format!("Function declared on line {} is invalid. Expected '{{', but received '{}'. Check your function arguments.", next.line, next.text));
                    }
                }
            }
            _ => ()
        }
        // Update symbol register
        self.last_symbol = next.symbol;
        return error_message;
    }
}

// -------------------- Grammar: Properties --------------------

struct GrammarProperties {
    is_valid: bool,
    done: bool,
    stage: u8,
    last_symbol: Symbol,
    p_list: Vec<properties::Properties>
}

impl GrammarProperties {
    fn new() -> GrammarProperties {
        GrammarProperties { 
            is_valid: true, 
            done: false,
            stage: 0,
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
            0 => {
                match next.symbol {
                    Symbol::DoubleColon => {
                        self.stage = 1;
                    },
                    _ => {
                        self.is_valid = false;
                        self.done = true;
                        error_message = Some(format!("Property list declared on line {} is invalid. Should be `#Properties :: A B C`.", next.line));
                    }
                }
            },
            1 => {
                match next.symbol {
                    Symbol::Value => {
                        match next.text.as_str() {
                            "Pure" => self.p_list.push(properties::Properties::Pure),
                            "Export" => self.p_list.push(properties::Properties::Export),
                            _ => {
                                self.is_valid = false;
                                self.done = true;
                                error_message = Some(format!("Property list declared on line {} is invalid. Unrecognized property {}. Valid properties are:\n{:?}", next.line, next.text, properties::PROPERTY_LIST));
                            }
                        }
                    },
                    Symbol::Newline => {
                        if self.p_list.len() == 0 {
                            println!("Warning: empty property list. A property list was declared on line {}, but no properties were provided.", next.line);
                        }
                        self.done = true;
                    },
                    _ => {
                        self.is_valid = false;
                        self.done = true;
                        error_message = Some(format!("Property list declared on line {} is invalid. Expected a valid property name or a new line, but received an unexpected token instead. The offending token is {}, which has symbol {:?}.", next.line, next.text, next.symbol));
                    }
                }
            },
            _ => ()
        }
        return error_message;
    }
}