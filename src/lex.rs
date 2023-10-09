/// Symbol defines what is recognized by the lexer
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Symbol {
    Value,
    FunctionDeclare,
    DoubleColon,
    RightArrow,
    EqualSign,
    DoubleEqualSign,
    Plus,
    Minus,
    Slash,
    Star,
    Hat,
    Gt,
    Lt,
    Gte,
    Lte,
    BraceLeft,
    BraceRight,
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
    RequirementsDeclaration,
    ContractPre,
    ContractPost,
    ContractInvariant,
    Let,
    Mut,
    TypeStr,
    TypeInt,
    TypeBool,
    TypeVoid
}

impl Symbol {
    fn identify(input: &str) -> Symbol {
        match input {
            "fn" => Symbol::FunctionDeclare,
            "::" => Symbol::DoubleColon,
            "->" => Symbol::RightArrow,
            "=" => Symbol::EqualSign,
            "==" => Symbol::DoubleEqualSign,
            "+" => Symbol::Plus,
            "-" => Symbol::Minus,
            "/" => Symbol::Slash,
            "*" => Symbol::Star,
            "^" => Symbol::Hat,
            ">" => Symbol::Gt,
            "<" => Symbol::Lt,
            ">=" => Symbol::Gte,
            "<=" => Symbol::Lte,
            "{" => Symbol::BraceLeft,
            "}" => Symbol::BraceRight,
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
            "#Requirements" => Symbol::RequirementsDeclaration,
            "#In" => Symbol::ContractPre,
            "#Out" => Symbol::ContractPost,
            "#Invariant" => Symbol::ContractInvariant,
            "let" => Symbol::Let,
            "mut" => Symbol::Mut,
            "str" => Symbol::TypeStr,
            "int" => Symbol::TypeInt,
            "bool" => Symbol::TypeBool,
            "void" => Symbol::TypeVoid,
            _ => Symbol::Value,
        }
    }
}

/// A token is a symbol and its context in the source code
#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub text: String,
    pub symbol: Symbol,
    pub line: usize,
}

impl Token {
    pub fn new(text: &str, line: usize) -> Token {
        Token {
            text: text.to_string(),
            symbol: Symbol::identify(text),
            line: line,
        }
    }
}

/// Process a code string and return a vector of tokens
pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    // Analyze line by line (delegates issue of deciding what constitutes a new line)
    for (index, line) in input.lines().enumerate() {
        // Split on some standard whitespace
        let words = line.split(&[' ', '\t', '\r']);
        for word in words {
            // Handle exceptions to the "partition by space" rule
            if word == "" {
                // Skip empty lines
                continue;
            } else if word.starts_with('(') || word.ends_with(')') {
                // Handle parenthesis
                let mut offset_start = 0usize;
                let mut offset_end = word.len();
                // ASSUME that '(' always appears at beginning, ')' appears at end
                for char in word.chars() {
                    if char == '(' {
                        offset_start += 1;
                        tokens.push(Token::new("(", index));
                    }
                    if char == ')' {
                        offset_end -= 1;
                        tokens.push(Token::new(")", index));
                    }
                }
                tokens.push(Token::new(&word[offset_start..offset_end], index));
            } else {
                // Default case
                tokens.push(Token::new(word, index));
            }
        }
        // Add new line separator
        tokens.push(Token::new("\n", index));
    }
    _ = tokens.pop();
    return tokens;
}

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
            Symbol::BraceLeft,
            Symbol::Newline,
            Symbol::Value,
            Symbol::Value,
            Symbol::Value,
            Symbol::Newline,
            Symbol::BraceRight,
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
            Symbol::Lt,
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
            Symbol::Value,
            Symbol::Value,
            Symbol::Value,
            Symbol::Value,
            Symbol::Value,
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
            Symbol::BraceLeft,
            Symbol::Newline,
            Symbol::PropertyDeclaration,
            Symbol::DoubleColon,
            Symbol::Value,
            Symbol::Value,
            Symbol::Newline,
            Symbol::Return,
            Symbol::Value,
            Symbol::Plus,
            Symbol::Value,
            Symbol::Newline,
            Symbol::BraceRight,
        ];
        let tokens = lex(program);
        let actual = tokens.iter().map(|t| t.symbol).collect::<Vec<Symbol>>();
        assert_eq!(actual, expected);
    }
}
