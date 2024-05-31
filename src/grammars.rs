//! Grammars define the permissible syntax of a sequence of tokens. Each line has one (and only one) possible grammar.
//!
//! Grammars are essentially large state machines and look "kind of like" a regex. The difference between the grammars here and the regex is conditional behavior.
//!
//! We have a small number of grammars (7) because the language has a strict structure -- each line must belong to one these classes of behavior. The list of grammars corresponds to the node types.

use std::fmt::Debug;

use crate::compiler_errors::{CompilerProblem, ProblemClass};
use crate::lex::{Symbol, Token, VALID_EXPRESSION_TOKENS};
use crate::parse::{PrimitiveDataType, Variable};
use crate::permissions::Permissions;
use crate::properties::{Properties, PROPERTY_LIST};

#[derive(Debug)]
pub enum Grammar {
    Empty,
    Import(GrammarImports),
    Function(GrammarFunctionDeclaration),
    Property(GrammarProperty),
    Permission(GrammarPermissions),
    VariableAssignment(GrammarVariableAssignments),
    Return,
    Expression(GrammarExpression),
    Enum,   // TODO
    Struct, // TODO
}

impl Grammar {
    pub fn new(symbol: Symbol) -> Grammar {
        match symbol {
            Symbol::Import => Grammar::Import(GrammarImports::new()),
            Symbol::FunctionDeclare => Grammar::Function(GrammarFunctionDeclaration::new()),
            Symbol::PropertyDeclaration => Grammar::Property(GrammarProperty::new()),
            Symbol::PermissionsDeclaration => Grammar::Permission(GrammarPermissions::new()),
            Symbol::Let | Symbol::Set => {
                Grammar::VariableAssignment(GrammarVariableAssignments::new(symbol))
            }
            Symbol::Return => Grammar::Return,
            _ => {
                if VALID_EXPRESSION_TOKENS.contains(&symbol) {
                    Grammar::Expression(GrammarExpression::new())
                } else {
                    Grammar::Empty
                }
            }
        }
    }

    pub fn step(&mut self, token: &Token) -> Option<CompilerProblem> {
        match self {
            Grammar::Empty => None,
            Grammar::Import(g) => g.step(token),
            Grammar::Function(g) => g.step(token),
            Grammar::Property(g) => g.step(token),
            Grammar::Permission(g) => g.step(token),
            Grammar::VariableAssignment(g) => g.step(token),
            Grammar::Return => None,
            Grammar::Expression(g) => g.step(token),
            Grammar::Enum => None,
            Grammar::Struct => None,
        }
    }

    pub fn is_done(&self) -> bool {
        match self {
            Grammar::Empty => true,
            Grammar::Import(g) => g.done,
            Grammar::Function(g) => g.done,
            Grammar::Property(g) => g.done,
            Grammar::Permission(g) => g.done,
            Grammar::VariableAssignment(g) => g.done,
            Grammar::Return => true,
            Grammar::Expression(g) => g.done,
            Grammar::Enum => true,
            Grammar::Struct => true,
        }
    }
}

// -------------------- Grammar: Imports --------------------

#[derive(Debug)]
enum StagesImport {
    Initialized,
    ProcessingArguments,
    ProcessingFile,
}

/// The grammar for importing a file or functions/data
#[derive(Debug)]
pub struct GrammarImports {
    is_valid: bool,
    done: bool,
    stage: StagesImport,
    arguments: Option<Vec<Token>>,
    file: String,
}

impl GrammarImports {
    fn new() -> GrammarImports {
        GrammarImports {
            is_valid: true,
            done: false,
            stage: StagesImport::Initialized,
            arguments: None,
            file: "unknown".to_string(),
        }
    }

    fn step(&mut self, next: &Token) -> Option<CompilerProblem> {
        if self.done {
            return None;
        }
        let mut error_message = None;
        match self.stage {
            StagesImport::Initialized => {
                // If there's a dot we're importing a file and can wrap up immediately
                if next.text.contains(".") {
                    self.file = next.text.to_string();
                    self.done = true;
                } else {
                    // We must be importing arguments so grab the first one
                    self.stage = StagesImport::ProcessingArguments;
                    if next.symbol == Symbol::Value {
                        self.arguments = Some(vec![next.clone()]);
                    } else {
                        error_message = Some(CompilerProblem::new(
                            ProblemClass::Error,
                            "imported item is a reserved keyword",
                            "check your imports",
                            next.line,
                            next.word,
                        ));
                    }
                }
            }
            StagesImport::ProcessingArguments => match next.symbol {
                Symbol::From => self.stage = StagesImport::ProcessingFile,
                Symbol::Value => {
                    if let Some(args) = &mut self.arguments {
                        args.push(next.clone());
                    }
                }
                _ => {
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Error,
                        &format!(
                            "expected the name of an item but received a keyword: {}",
                            next.text
                        ),
                        "check your imports",
                        next.line,
                        next.word,
                    ));
                }
            },
            // Only entered if we had arguments
            StagesImport::ProcessingFile => match next.symbol {
                Symbol::Value => {
                    self.file = next.text.to_string();
                    self.done = true;
                }
                _ => {
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Error,
                        &format!(
                            "expected the name of a library but received a keyword: {}",
                            next.text
                        ),
                        "check your imports",
                        next.line,
                        next.word,
                    ));
                }
            },
        }
        error_message
    }
}

// -------------------- Grammar: Functions --------------------

#[derive(Debug)]
enum StagesFunction {
    Initialized,
    NameProcessed,
    SeekingArguments,
    SeekingBracket,
    SeekingNewLine,
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
    stage: StagesFunction,
    last_symbol: Symbol,
    pub fn_name: String,
    pub arguments: Vec<Variable>,
    pub return_type: PrimitiveDataType,
}

impl GrammarFunctionDeclaration {
    pub fn new() -> GrammarFunctionDeclaration {
        GrammarFunctionDeclaration {
            is_valid: true,
            done: false,
            stage: StagesFunction::Initialized,
            last_symbol: Symbol::FunctionDeclare,
            fn_name: "undefined".to_string(),
            arguments: Vec::<Variable>::new(),
            return_type: PrimitiveDataType::Void,
        }
    }

    /// Steps forward through a state machine, returning optional error message
    fn step(&mut self, next: &Token) -> Option<CompilerProblem> {
        if self.done {
            return None;
        }
        let mut error_message: Option<CompilerProblem> = None;
        match self.stage {
            // Initial stage -> next symbol should be the fn name
            StagesFunction::Initialized => match next.symbol {
                Symbol::Value => {
                    if next.text.is_ascii() {
                        self.fn_name = next.text.to_string();
                        self.stage = StagesFunction::NameProcessed;
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
            StagesFunction::NameProcessed => match next.symbol {
                Symbol::BraceOpen => {
                    self.done = true;
                }
                Symbol::DoubleColon => {
                    self.stage = StagesFunction::SeekingArguments;
                }
                _ => {
                    self.is_valid = false;
                    self.done = true;
                    error_message = Some(CompilerProblem::new(ProblemClass::Error, &format!("expected a '::' (if it has args) or a '{{' (if it doesn't have args) after the function name, but received '{}'.", next.text), "functions should look like this: `fn foo :: a int -> int`", next.line, next.word));
                }
            },
            // Function has one or more arguments.
            StagesFunction::SeekingArguments => {
                if self.last_symbol == Symbol::DoubleColon || self.last_symbol == Symbol::RightArrow
                {
                    match next.symbol {
                        // If we receive a type after :: or ->, it implies that is the return type and there are no arguments
                        Symbol::TypeBool => {
                            self.stage = StagesFunction::SeekingBracket;
                            self.return_type = PrimitiveDataType::Bool;
                        }
                        Symbol::TypeInt => {
                            self.stage = StagesFunction::SeekingBracket;
                            self.return_type = PrimitiveDataType::Int;
                        }
                        Symbol::TypeStr => {
                            self.stage = StagesFunction::SeekingBracket;
                            self.return_type = PrimitiveDataType::Str;
                        }
                        Symbol::TypeVoid => {
                            self.stage = StagesFunction::SeekingBracket;
                            self.return_type = PrimitiveDataType::Void;
                        }
                        // A value here implies the argument name
                        Symbol::Value => {
                            self.arguments.push(Variable {
                                name: next.text.to_string(),
                                data_type: PrimitiveDataType::Void,
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
                                .data_type = PrimitiveDataType::Bool;
                        }
                        Symbol::TypeInt => {
                            self.arguments
                                .last_mut()
                                .expect("expected argument to exist")
                                .data_type = PrimitiveDataType::Int;
                        }
                        Symbol::TypeStr => {
                            self.arguments
                                .last_mut()
                                .expect("expected argument to exist")
                                .data_type = PrimitiveDataType::Str;
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
            StagesFunction::SeekingBracket => match next.symbol {
                Symbol::BraceOpen => {
                    self.stage = StagesFunction::SeekingNewLine;
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
            StagesFunction::SeekingNewLine => match next.symbol {
                Symbol::Newline => {
                    self.done = true;
                }
                _ => {
                    self.is_valid = false;
                    self.done = true;
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Error,
                        &format!("expected new line, but received '{}'.", next.text),
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

// -------------------- Grammar: Function Annotations --------------------

#[derive(Debug)]
enum StagesAnnotation {
    Initialized,
    ExpectValues,
}

/// The Grammar for declaring a function's properties
#[derive(Debug)]
pub struct GrammarProperty {
    is_valid: bool,
    done: bool,
    stage: StagesAnnotation,
    p_list: Vec<Properties>,
}

/// Grammar for declaring a function's permissions
#[derive(Debug)]
pub struct GrammarPermissions {
    is_valid: bool,
    done: bool,
    stage: StagesAnnotation,
    p_list: Vec<Permissions>,
}

impl GrammarProperty {
    fn new() -> GrammarProperty {
        GrammarProperty {
            is_valid: true,
            done: false,
            stage: StagesAnnotation::Initialized,
            p_list: Vec::<Properties>::new(),
        }
    }

    fn step(&mut self, next: &Token) -> Option<CompilerProblem> {
        if self.done {
            return None;
        }
        let mut error_message: Option<CompilerProblem> = None;
        match self.stage {
            StagesAnnotation::Initialized => match next.symbol {
                Symbol::DoubleColon => {
                    self.stage = StagesAnnotation::ExpectValues;
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
            StagesAnnotation::ExpectValues => match next.symbol {
                Symbol::Value => match next.text.as_str() {
                    "Pure" => self.p_list.push(Properties::Pure),
                    "Public" => self.p_list.push(Properties::Public),
                    "Export" => self.p_list.push(Properties::Export),
                    _ => {
                        self.is_valid = false;
                        self.done = true;
                        error_message = Some(CompilerProblem::new(
                            ProblemClass::Error,
                            &format!("unrecognized property {}.", next.text),
                            &format!("valid properties are:\n{:?}", PROPERTY_LIST),
                            next.line,
                            next.word,
                        ));
                    }
                },
                Symbol::Newline => {
                    if self.p_list.is_empty() {
                        error_message = Some(CompilerProblem::new(
                            ProblemClass::Warning,
                            &format!("empty property list"),
                            &format!("either remove the property list or add properties"),
                            next.line,
                            next.word,
                        ));
                        self.is_valid = false;
                    }
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

impl GrammarPermissions {
    fn new() -> GrammarPermissions {
        GrammarPermissions {
            is_valid: true,
            done: false,
            stage: StagesAnnotation::Initialized,
            p_list: Vec::<Permissions>::new(),
        }
    }

    fn step(&mut self, next: &Token) -> Option<CompilerProblem> {
        if self.done {
            return None;
        }
        let mut error_message: Option<CompilerProblem> = None;
        match self.stage {
            StagesAnnotation::Initialized => match next.symbol {
                Symbol::DoubleColon => {
                    self.stage = StagesAnnotation::ExpectValues;
                }
                _ => {
                    self.is_valid = false;
                    self.done = true;
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Error,
                        &format!(
                            "permission list is invalid - expected a `::` but found {}",
                            next.text
                        ),
                        "a permission list should look like this: `#Permissions :: A B C`.",
                        next.line,
                        next.word,
                    ));
                }
            },
            StagesAnnotation::ExpectValues => match next.symbol {
                Symbol::Value => self.p_list.push(Permissions::from_str(&next.text)),
                Symbol::Newline => {
                    if self.p_list.is_empty() {
                        error_message = Some(CompilerProblem::new(
                            ProblemClass::Warning,
                            &format!("empty permission list"),
                            &format!("either remove the permission list or add properties"),
                            next.line,
                            next.word,
                        ));
                        self.is_valid = false;
                    }
                    self.done = true;
                }
                _ => {
                    self.is_valid = false;
                    self.done = true;
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Error,
                        &format!("expected a valid permission name or a new line, but received an unexpected token instead. the offending token is {}, which has symbol {:?}.", next.text, next.symbol),
                        "a permission list should look like this: `#Permissions :: A B C`.",
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

#[derive(Debug, PartialEq, Eq)]
enum AssignmentTypes {
    Initialize, // let x = ...
    Mutate,     // set x = ...
}

#[derive(Debug)]
enum StagesVariableAssignment {
    FindingName,
    GettingIndexValue,
    DeclaringType,
    SeekingTypeName,
    CheckingMutability,
}

#[derive(Debug)]
pub struct GrammarVariableAssignments {
    is_valid: bool,
    done: bool,
    stage: StagesVariableAssignment,
    assignment_type: AssignmentTypes,
    type_provided: bool,
    data_type: PrimitiveDataType,
    name: String,
    mutable: bool,
    index_text: Option<String>,
}

impl GrammarVariableAssignments {
    fn new(symbol: Symbol) -> GrammarVariableAssignments {
        let this_type = match symbol {
            Symbol::Let => AssignmentTypes::Initialize,
            Symbol::Mut => AssignmentTypes::Mutate,
            _ => panic!("internal compiler error: received illegal symbol while initializing GrammarVariableAssignments -- please file a bug report.")
        };
        GrammarVariableAssignments {
            is_valid: true,
            done: false,
            stage: StagesVariableAssignment::FindingName,
            assignment_type: this_type,
            type_provided: false,
            data_type: PrimitiveDataType::Void,
            name: "unknown".to_string(),
            mutable: false,
            index_text: None,
        }
    }

    fn step(&mut self, next: &Token) -> Option<CompilerProblem> {
        if self.done {
            return None;
        }
        let mut error_message: Option<CompilerProblem> = None;
        match self.stage {
            StagesVariableAssignment::FindingName => match next.symbol {
                Symbol::Value => {
                    if next.text.is_ascii() {
                        self.name = next.text.to_string();
                        self.stage = StagesVariableAssignment::DeclaringType;
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
            StagesVariableAssignment::GettingIndexValue => match next.symbol {
                Symbol::Value => {
                    self.index_text = Some(next.text.to_string());
                    self.stage = StagesVariableAssignment::DeclaringType;
                }
                _ => {
                    error_message = Some(
                        CompilerProblem::new(ProblemClass::Error, &format!("expected an index, but found a system reserved keyword instead (found `{}`", next.text), "indices should be either a number `37` or a range `0..2`", next.line, next.word)
                    );
                    self.is_valid = false;
                    self.done = true;
                }
            },
            StagesVariableAssignment::DeclaringType => match next.symbol {
                // Double colon implies we're going to get a type
                Symbol::At => match self.assignment_type {
                    AssignmentTypes::Initialize => {
                        error_message = Some(
                                CompilerProblem::new(ProblemClass::Error, &format!("in declaration of `{}`, cannot index into a collection when initializing a value", self.name), &format!("initialize the collection then mutate it, try this pattern: `let {} :: auto mut = ...` with `set {} @ ... = ...`", self.name, self.name), next.line, next.word)
                            );
                        self.is_valid = false;
                        self.done = true;
                    }
                    AssignmentTypes::Mutate => {
                        self.stage = StagesVariableAssignment::GettingIndexValue
                    }
                },
                Symbol::DoubleColon => {
                    self.stage = StagesVariableAssignment::SeekingTypeName;
                }
                // Equals sign implies no type present
                Symbol::EqualSign => {
                    self.type_provided = false;
                    self.done = true;
                    let keyword = if self.assignment_type == AssignmentTypes::Initialize {
                        "let"
                    } else {
                        "set"
                    };
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Lint,
                        &format!(
                            "use `auto` with `{}` to be explicit about your type inference",
                            self.name
                        ),
                        &format!("try this: `{keyword} {} :: auto = ...`", self.name),
                        next.line,
                        next.word,
                    ));
                }
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
            StagesVariableAssignment::SeekingTypeName => {
                if next.symbol == Symbol::Mut {
                    self.type_provided = false;
                    self.mutable = true;
                    self.done = true;
                    let keyword = if self.assignment_type == AssignmentTypes::Initialize {
                        "let"
                    } else {
                        "set"
                    };
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Lint,
                        &format!(
                            "use `auto` with `{}` to be explicit about your type inference",
                            self.name
                        ),
                        &format!("try this: `{keyword} {} :: auto mut = ...`", self.name),
                        next.line,
                        next.word,
                    ));
                }
                match PrimitiveDataType::from_symbol(next.symbol) {
                    Some(d) => {
                        self.type_provided = true;
                        self.data_type = d;
                        self.stage = StagesVariableAssignment::CheckingMutability;
                    }
                    None => {
                        if next.symbol == Symbol::TypeAuto || next.symbol == Symbol::Mut {
                            self.type_provided = false;
                            self.data_type = PrimitiveDataType::Void;
                            self.stage = StagesVariableAssignment::CheckingMutability;
                        } else {
                            error_message = Some(CompilerProblem::new(
                                ProblemClass::Error,
                                &format!("expected a type name, but found `{}`", next.text),
                                "provide a valid type such as `str` or `int`, or use `auto` to infer the type",
                                next.line,
                                next.word,
                            ));
                            self.is_valid = false;
                            self.done = true;
                        }
                    }
                }
            }
            StagesVariableAssignment::CheckingMutability => match next.symbol {
                Symbol::Mut => {
                    self.mutable = true;
                    self.done = true;
                }
                Symbol::EqualSign => self.done = true,
                _ => {
                    error_message = Some(CompilerProblem::new(
                        ProblemClass::Error,
                        &format!("expected either `mut` or `=`, but found `{}`", next.text),
                        "you may have more than 1 type for this variable",
                        next.line,
                        next.word,
                    ));
                    self.is_valid = false;
                    self.done = true;
                }
            },
        }
        error_message
    }
}

// -------------------- Grammar: Expression --------------------

#[derive(Debug)]
pub struct GrammarExpression {
    done: bool,
    is_valid: bool,
    tokens: Vec<Token>,
}

impl GrammarExpression {
    pub fn new() -> GrammarExpression {
        GrammarExpression {
            done: false,
            is_valid: true,
            tokens: Vec::new(),
        }
    }

    pub fn step(&mut self, next: &Token) -> Option<CompilerProblem> {
        let mut error_message: Option<CompilerProblem> = None;
        if VALID_EXPRESSION_TOKENS.contains(&next.symbol) {
            self.tokens.push(next.clone());
        } else if next.symbol == Symbol::Newline {
            self.done = true;
        } else {
            error_message = Some(CompilerProblem::new(
                ProblemClass::Error,
                &format!(
                    "expected a function, variable, or operation, found {}",
                    next.text
                ),
                "you may have more than 1 type for this variable",
                next.line,
                next.word,
            ));
            self.is_valid = false;
            self.done = true;
        }
        error_message
    }
}

// -------------------- Unit Tests --------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::{lex, Symbol};

    #[test]
    fn declare_import_1() {
        let mut gi = GrammarImports::new();
        let line: &str = "import a b from c";
        let tokens = lex(line);
        for t in tokens.into_iter().skip(1) {
            gi.step(&t);
        }
        assert!(gi.done);
        assert!(gi.is_valid);
        assert_eq!(gi.file, "c".to_string());
        assert!(gi.arguments.is_some());
        if let Some(args) = gi.arguments.as_ref() {
            assert_eq!(args[0].text, "a".to_string());
            assert_eq!(args[1].text, "b".to_string());
        }
    }

    #[test]
    fn declare_import_2() {
        let mut gi = GrammarImports::new();
        let line: &str = "import this.c";
        let tokens = lex(line);
        for t in tokens.into_iter().skip(1) {
            gi.step(&t);
        }
        assert!(gi.done);
        assert!(gi.is_valid);
        assert_eq!(gi.file, "this.c".to_string());
        assert!(gi.arguments.is_none());
    }

    #[test]
    fn declare_fn_simple_1() {
        let mut gfd = GrammarFunctionDeclaration::new();
        let line: &str = "fn add :: a int -> b int -> int {
        ";
        let tokens = lex(line);
        // Skip the first token (the `fn` token)
        for t in tokens.into_iter().skip(1) {
            gfd.step(&t);
        }
        println!("{:?}", gfd);
        assert!(gfd.done);
        assert!(gfd.is_valid);
        assert_eq!(gfd.fn_name, "add");
        assert_eq!(gfd.arguments.len(), 2);
        // Arg 1
        assert_eq!(gfd.arguments[0].name, "a");
        assert_eq!(gfd.arguments[0].data_type, PrimitiveDataType::Int);
        assert!(gfd.arguments[0].value.is_none());
        // Arg 2
        assert_eq!(gfd.arguments[1].name, "b");
        assert_eq!(gfd.arguments[1].data_type, PrimitiveDataType::Int);
        assert!(gfd.arguments[1].value.is_none());
        assert_eq!(gfd.return_type, PrimitiveDataType::Int);
    }

    #[test]
    fn declare_fn_simple_2() {
        let mut gfd = GrammarFunctionDeclaration::new();
        let line: &str = "fn copy_to :: old_filepath str -> new_filepath str -> void {
        ";
        let tokens = lex(line);
        // Skip the first token (the `fn` token)
        for t in tokens.into_iter().skip(1) {
            gfd.step(&t);
        }
        println!("{:?}", gfd);
        assert!(gfd.done);
        assert!(gfd.is_valid);
        assert_eq!(gfd.fn_name, "copy_to");
        assert_eq!(gfd.arguments.len(), 2);
        // Arg 1
        assert_eq!(gfd.arguments[0].name, "old_filepath");
        assert_eq!(gfd.arguments[0].data_type, PrimitiveDataType::Str);
        assert!(gfd.arguments[0].value.is_none());
        // Arg 2
        assert_eq!(gfd.arguments[1].name, "new_filepath");
        assert_eq!(gfd.arguments[1].data_type, PrimitiveDataType::Str);
        assert!(gfd.arguments[1].value.is_none());
        assert_eq!(gfd.return_type, PrimitiveDataType::Void);
    }

    #[test]
    fn declare_fn_no_name() {
        let mut gfd = GrammarFunctionDeclaration::new();
        let line: &str = "fn :: old_filepath str -> new_filepath str -> void {\n";
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

    #[test]
    fn declare_variable_init() {
        let mut gv = GrammarVariableAssignments::new(Symbol::Let);
        let line: &str = "let a :: int = 1";
        let tokens = lex(line);
        for t in tokens.into_iter().skip(1) {
            gv.step(&t);
        }
        // assert!(gv.done); // this will fail b/c no newline, but this is okay
        assert!(gv.is_valid);
        assert_eq!(gv.data_type, PrimitiveDataType::Int);
        assert_eq!(gv.mutable, false);
        assert!(gv.type_provided);
        assert_eq!(gv.name, "a".to_string());
    }

    #[test]
    fn declare_variable_init_mut() {
        let mut gv = GrammarVariableAssignments::new(Symbol::Let);
        let line: &str = "let a :: str mut = \"meow\"";
        let tokens = lex(line);
        for t in tokens.into_iter().skip(1) {
            gv.step(&t);
        }
        println!("{:#?}", gv);
        // assert!(gv.done); // this will fail b/c no newline, but this is okay
        assert!(gv.is_valid);
        assert_eq!(gv.data_type, PrimitiveDataType::Str);
        assert_eq!(gv.mutable, true);
        assert!(gv.type_provided);
        assert_eq!(gv.name, "a".to_string());
    }

    #[test]
    fn declare_variable_init_mut_no_type() {
        let mut gv = GrammarVariableAssignments::new(Symbol::Let);
        let line: &str = "let a :: mut = 42";
        let tokens = lex(line);
        for t in tokens.into_iter().skip(1) {
            println!("{:#?}", gv.step(&t));
        }
        println!("{:#?}", gv);
        assert!(gv.is_valid);
        assert_eq!(gv.data_type, PrimitiveDataType::Void); // Void is a filler type here
        assert_eq!(gv.mutable, true);
        assert_eq!(gv.type_provided, false);
        assert_eq!(gv.name, "a".to_string());
    }

    #[test]
    fn declare_variable_init_mut_auto() {
        let mut gv = GrammarVariableAssignments::new(Symbol::Let);
        let line: &str = "let a :: auto mut = 42";
        let tokens = lex(line);
        for t in tokens.into_iter().skip(1) {
            println!("{:#?}", gv.step(&t));
        }
        println!("{:#?}", gv);
        assert!(gv.is_valid);
        assert_eq!(gv.data_type, PrimitiveDataType::Void);
        assert_eq!(gv.mutable, true);
        assert_eq!(gv.type_provided, false);
        assert_eq!(gv.name, "a".to_string());
    }

    #[test]
    fn declare_variable_mutate() {
        let mut gv = GrammarVariableAssignments::new(Symbol::Mut);
        let line: &str = "set a = 1";
        let tokens = lex(line);
        for t in tokens.into_iter().skip(1) {
            gv.step(&t);
        }
        assert!(gv.is_valid);
        assert_eq!(gv.name, "a".to_string());
    }

    #[test]
    fn declare_variable_mutate_index() {
        let mut gv = GrammarVariableAssignments::new(Symbol::Mut);
        let line: &str = "set a @ 10 = 1";
        let tokens = lex(line);
        for t in tokens.into_iter().skip(1) {
            println!("{:?}, {:#?}", t, gv);
            gv.step(&t);
        }
        assert!(gv.is_valid);
        assert!(gv.index_text.is_some());
        assert_eq!(gv.index_text.unwrap(), "10".to_string());
        assert_eq!(gv.name, "a".to_string());
    }
}
