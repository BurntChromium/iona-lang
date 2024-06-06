//! This is a (large) submodule of the parser dedicated to expression parsing

enum Operator {
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

enum Expression {
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

enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Symbol(String),
}

enum Object {
    Operation(Expression),
    Value(Literal),
}
