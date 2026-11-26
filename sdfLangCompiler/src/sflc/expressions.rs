#[derive(Debug)]
pub enum FunctionOperations {
    Minus(Box<Expression>),
    Absolute(Box<Expression>),
    Cos(Box<Expression>),
    Sin(Box<Expression>),
    Tan(Box<Expression>),
    Ln(Box<Expression>),
    Log2(Box<Expression>),
    SquareRoot(Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),
    SmoothMin(Box<Expression>, Box<Expression>, Box<Expression>),
    SmoothMax(Box<Expression>, Box<Expression>, Box<Expression>),
    Mix(Box<Expression>, Box<Expression>, Box<Expression>),
    Clamp(Box<Expression>, Box<Expression>, Box<Expression>),
}

#[derive(Debug)]
pub enum BinaryOperations {
    Add,
    Subtract,
    Multiply,
    Divide,
    //`left` = Base | `right` = index -- sqrt(base, index)
    Power,
    //`left` = Base | `right` = exponent -- pow(base, exponent)
}

#[derive(Debug)]
pub enum Expression {
    Function(FunctionOperations),

    Binary {
        left: Box<Expression>,
        operator: BinaryOperations,
        right: Box<Expression>,
    },
    Variable(String),
    Number(f32),
}
