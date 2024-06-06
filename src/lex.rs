/// Symbol defines what is recognized by the lexer
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Symbol {
    Value, // needs further evaluation
    FunctionDeclare,
    DoubleColon,
    RightArrow,
    EqualSign,
    DoubleEqualSign,
    OpPlus,
    OpMinus,
    OpDiv,
    OpMul,
    OpExp,
    OpGt,
    OpLt,
    OpGte,
    OpLte,
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    Return,
    Import,
    From,
    Set,
    Get,
    If,
    Else,
    Comment,
    Newline,
    PropertyDeclaration,
    PermissionsDeclaration,
    ContractPre,
    ContractPost,
    ContractInvariant,
    Let,
    Mut,
    TypeStr,
    TypeInt,
    TypeBool,
    TypeVoid,
    TypeFloat,
    TypeAuto,
    At,
}

impl Symbol {
    fn identify(input: &str) -> Symbol {
        match input {
            "fn" => Symbol::FunctionDeclare,
            "::" => Symbol::DoubleColon,
            "->" => Symbol::RightArrow,
            "=" => Symbol::EqualSign,
            "==" => Symbol::DoubleEqualSign,
            "+" => Symbol::OpPlus,
            "-" => Symbol::OpMinus,
            "/" => Symbol::OpDiv,
            "*" => Symbol::OpMul,
            "^" => Symbol::OpExp,
            ">" => Symbol::OpGt,
            "<" => Symbol::OpLt,
            ">=" => Symbol::OpGte,
            "<=" => Symbol::OpLte,
            "(" => Symbol::ParenOpen,
            ")" => Symbol::ParenClose,
            "{" => Symbol::BraceOpen,
            "}" => Symbol::BraceClose,
            "return" => Symbol::Return,
            "import" => Symbol::Import,
            "from" => Symbol::From,
            "set" => Symbol::Set,
            "get" => Symbol::Get,
            "if" => Symbol::If,
            "else" => Symbol::Else,
            "//" => Symbol::Comment,
            "\n" => Symbol::Newline,
            "#Properties" => Symbol::PropertyDeclaration,
            "#Permissions" => Symbol::PermissionsDeclaration,
            "#In" => Symbol::ContractPre,
            "#Out" => Symbol::ContractPost,
            "#Invariant" => Symbol::ContractInvariant,
            "let" => Symbol::Let,
            "mut" => Symbol::Mut,
            "str" => Symbol::TypeStr,
            "int" => Symbol::TypeInt,
            "bool" => Symbol::TypeBool,
            "float" => Symbol::TypeFloat,
            "void" => Symbol::TypeVoid,
            "auto" => Symbol::TypeAuto,
            "@" => Symbol::At,
            _ => Symbol::Value,
        }
    }
}

/// These symbols are banned on the RHS of any expression
pub const BANNED_RHS_SYMBOLS: [Symbol; 18] = [
    Symbol::FunctionDeclare,
    Symbol::DoubleColon,
    Symbol::Return,
    Symbol::Import,
    Symbol::From,
    Symbol::PropertyDeclaration,
    Symbol::PermissionsDeclaration,
    Symbol::ContractPre,
    Symbol::ContractPost,
    Symbol::ContractInvariant,
    Symbol::Let,
    Symbol::Mut,
    Symbol::TypeBool,
    Symbol::TypeFloat,
    Symbol::TypeInt,
    Symbol::TypeStr,
    Symbol::TypeVoid,
    Symbol::TypeAuto,
];

pub const VALID_EXPRESSION_TOKENS: [Symbol; 13] = [
    Symbol::Value,
    Symbol::OpPlus,
    Symbol::OpMinus,
    Symbol::OpMul,
    Symbol::OpDiv,
    Symbol::OpExp,
    Symbol::OpGt,
    Symbol::OpLt,
    Symbol::OpGte,
    Symbol::OpLte,
    Symbol::At,
    Symbol::ParenOpen,
    Symbol::ParenClose,
];

/// A token is a symbol and its context in the source code
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub text: String,
    pub symbol: Symbol,
    pub line: usize,
    pub word: usize,
}

impl Token {
    pub fn new(text: &str, line: usize, word: usize) -> Token {
        Token {
            text: text.to_string(),
            symbol: Symbol::identify(text),
            line,
            word,
        }
    }
}

/// Process a code string and return a vector of tokens
pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    // Analyze line by line (delegates issue of deciding what constitutes a new line)
    for (line_index, line) in input.lines().enumerate() {
        // Split on some standard whitespace
        let words = line.split(&[' ', '\t', '\r']);
        // Handle special cases w.r.t. line breaks
        let mut words_p = words.clone().peekable();
        // Skip commented out lines
        if *words_p.peek().unwrap_or(&"\n") == "//" {
            tokens.push(Token::new("//", line_index, 0));
            tokens.push(Token::new("\n", line_index, 0));
            continue;
        }
        // Using `for (word_index, word) in words.enumerate()` gives the wrong indices
        let mut word_index: usize = 0;
        for word in words {
            // Handle exceptions to the "partition by space" rule
            if word.is_empty() {
                // Skip empty lines
                continue;
            } else if (word.starts_with('(') || word.ends_with(')')) && word.len() > 1 {
                // Handle parenthesis
                let mut offset_start = 0usize;
                let mut offset_end = word.len();
                let mut deferred_closing_parens = 0usize;
                // ASSUME that '(' always appears at beginning, ')' appears at end
                for char in word.chars() {
                    if char == '(' {
                        offset_start += 1;
                        tokens.push(Token::new("(", line_index, word_index));
                        word_index += 1;
                    }
                    if char == ')' {
                        offset_end -= 1;
                        deferred_closing_parens += 1;
                    }
                }
                // Push that word stripped of parens
                tokens.push(Token::new(
                    &word[offset_start..offset_end],
                    line_index,
                    word_index,
                ));
                // Push any trailing '('s
                word_index += 1;
                for _ in 0..deferred_closing_parens {
                    tokens.push(Token::new(")", line_index, word_index));
                    word_index += 1;
                }
            } else {
                // Default case
                tokens.push(Token::new(word, line_index, word_index));
                word_index += 1;
            }
        }
        // Add new line separator token
        if let Some(t) = tokens.last() {
            tokens.push(Token::new("\n", line_index, t.word + 1));
        } else {
            tokens.push(Token::new("\n", line_index, 0));
        }
    }
    // Pop the trailing newline we inserted
    _ = tokens.pop();
    tokens
}

// -------------------- Unit Tests --------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import() {
        let program: &str = "import read write from std.files";
        let expected: Vec<Symbol> = vec![
            Symbol::Import,
            Symbol::Value,
            Symbol::Value,
            Symbol::From,
            Symbol::Value,
        ];
        let tokens = lex(program);
        let actual = tokens.iter().map(|t| t.symbol).collect::<Vec<Symbol>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn hello_world() {
        let program: &str = "fn main {
            println \"Hello, world\"
        }";
        let expected: Vec<Symbol> = vec![
            Symbol::FunctionDeclare,
            Symbol::Value,
            Symbol::BraceOpen,
            Symbol::Newline,
            Symbol::Value,
            Symbol::Value,
            Symbol::Value,
            Symbol::Newline,
            Symbol::BraceClose,
        ];
        let tokens = lex(program);
        let actual = tokens.iter().map(|t| t.symbol).collect::<Vec<Symbol>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn partial_contract_pre() {
        let program: &str = "#In :: n < 0 -> \"n must be at least 0\"";
        let expected: Vec<Symbol> = vec![
            Symbol::ContractPre,
            Symbol::DoubleColon,
            Symbol::Value,
            Symbol::OpLt,
            Symbol::Value,
            Symbol::RightArrow,
            Symbol::Value,
            Symbol::Value,
            Symbol::Value,
            Symbol::Value,
            Symbol::Value,
            Symbol::Value,
        ];
        let tokens = lex(program);
        let actual = tokens.iter().map(|t| t.symbol).collect::<Vec<Symbol>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn basic_math_ops() {
        let program: &str = "// This function adds two numbers
        fn add :: a int -> b int -> int {
            #Properties :: Pure Export
            return a + b
        }";
        let expected: Vec<Symbol> = vec![
            Symbol::Comment,
            Symbol::Newline,
            Symbol::FunctionDeclare,
            Symbol::Value,
            Symbol::DoubleColon,
            Symbol::Value,
            Symbol::TypeInt,
            Symbol::RightArrow,
            Symbol::Value,
            Symbol::TypeInt,
            Symbol::RightArrow,
            Symbol::TypeInt,
            Symbol::BraceOpen,
            Symbol::Newline,
            Symbol::PropertyDeclaration,
            Symbol::DoubleColon,
            Symbol::Value,
            Symbol::Value,
            Symbol::Newline,
            Symbol::Return,
            Symbol::Value,
            Symbol::OpPlus,
            Symbol::Value,
            Symbol::Newline,
            Symbol::BraceClose,
        ];
        let tokens = lex(program);
        let actual = tokens.iter().map(|t| t.symbol).collect::<Vec<Symbol>>();
        assert_eq!(actual, expected);
        assert_eq!(tokens[2].line, 1);
        assert_eq!(tokens[2].word, 0);
        assert_eq!(tokens[3].line, 1);
        assert_eq!(tokens[3].word, 1);
    }
}
