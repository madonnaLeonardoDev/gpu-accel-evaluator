use crate::lexer::Tokens;

enum UnaryOperations {
    Minus,
    Absolute,
    Cos,
    Sin,
    Tan,
    Ln,
    Log2,
    Log
}

enum BinaryOperations {
    Add,
    Subtract,
    Multiply,
    Divide,
    SquareRoot,
    //`left` = Base | `right` = index -- sqrt(base, index)
    Power,
    //`left` = Base | `right` = exponent -- pow(base, exponent)
}

enum Expression {
    Unary{
        operation: UnaryOperations,
        expression: Box<Expression>
    },

    Binary{
        left: Box<Expression>,
        operator: BinaryOperations,
        right: Box<Expression>
    },
    DynVariable(String),
    StaticVariable(String),
    Number(f32)
}

struct Parser<'a>{
    tokens: &'a [Tokens],
    pointer: usize
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Tokens]) -> Self {
        Self {
            pointer: 0,
            tokens
        }
    }
}