use crate::lexer::Tokens;

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
    Clamp(Box<Expression>, Box<Expression>, Box<Expression>)
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

    Binary{
        left: Box<Expression>,
        operator: BinaryOperations,
        right: Box<Expression>
    },
    Variable(String),
    Number(f32)
}

pub struct Parser<'a>{
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

    pub fn parse_tokens(&mut self) -> Result<Expression, String> {
        let expr = self.parse_additive()?;

        if !matches!(self.peek(), Tokens::Eof) {
            return Err(format!("Unexpected token remaining at index {}", self.pointer))
        }

        Ok(expr)
    }

    fn peek(&mut self) -> &Tokens {
        self.tokens.get(self.pointer).unwrap_or(&Tokens::Eof)
    }

    fn advance(&mut self) -> &Tokens {
        let tok = self.tokens.get(self.pointer).unwrap_or(&Tokens::Eof);
        
        if *tok != Tokens::Eof {
            self.pointer += 1;
        }
        tok
    }

    fn parse_additive(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_multiplication()?;

        while matches!(self.peek(), Tokens::Plus | Tokens::Minus ) {
            let op = match self.advance() {
                Tokens::Plus => BinaryOperations::Add,
                Tokens::Minus => BinaryOperations::Subtract,
                _ => unreachable!()
            };

            let right = self.parse_multiplication()?;

            left = Expression::Binary { left: Box::new(left), operator: op, right: Box::new(right) };
        }

        Ok(left)
    } 

    fn parse_multiplication(&mut self)-> Result<Expression, String> {
        let mut left = self.parse_power()?;

        while matches!(self.peek(), Tokens::Times | Tokens::Divide) {
            let op = match self.advance() {
                Tokens::Times => BinaryOperations::Multiply,
                Tokens::Divide => BinaryOperations::Divide,
                _=> unreachable!()
            };

            let right = self.parse_power()?;

            left = Expression::Binary { left: Box::new(left), operator: op, right: Box::new(right) };
        }

        Ok(left)
    }

    fn parse_power(&mut self)-> Result<Expression, String> {
        let mut left = self.parse_unary()?;

        while matches!(self.peek(), Tokens::Power){
            self.advance();
            let op = BinaryOperations::Power;

            let right = self.parse_power()?;

            left = Expression::Binary { left: Box::new(left), operator: op, right: Box::new(right) }
        }

        Ok(left)
    }

    fn parse_unary(&mut self)-> Result<Expression, String> {
        if matches!(self.peek(), Tokens::Minus){
            self.advance();

            let expr = self.parse_unary()?;

            let op = FunctionOperations::Minus(Box::new(expr));

            return Ok(Expression::Function(op))
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        match self.advance() {
            Tokens::Number(n) => {
                return Ok(Expression::Number(*n))
            },
            Tokens::OpenPar => {
                let expr = self.parse_additive()?;

                if !matches!(self.peek(), Tokens::ClosedPar) {
                    return Err("Expected ')'".to_string())
                }

                self.advance();

                return Ok(expr)
            },
            Tokens::Identifier(name ) => {
                let name = name.clone();

                if matches!(self.peek(), Tokens::OpenPar) {
                    self.advance();
                    
                    let expected_args = match name.as_str() {
                        "smoothmin" | "smoothmax" | "clamp" | "mix" => 3,
                        "mod" => 2,
                        "abs" | "cos" | "sin" | "tan" | "ln" | "log2" | "sqrt" => 1,
                        _=> return Err(format!("Unkown function: '{}'", name))
                    };

                    let mut args: Vec<Expression> = Vec::with_capacity(expected_args);

                    if !matches!(self.peek(), Tokens::ClosedPar) {
                        args.push(self.parse_additive()?);

                        while matches!(self.peek(), Tokens::Comma){
                            self.advance();
                            args.push(self.parse_additive()?);
                        }
                    }

                    if !matches!(self.peek(), Tokens::ClosedPar) {
                        return Err("Expected ')'".to_string());
                    }

                    self.advance();

                    if args.len() != expected_args {
                        return Err(format!("Function: {} expects {} args but got {}", name, expected_args, args.len()))
                    }

                    let mut iter = args.into_iter();

                    let op = match name.as_str() {
                    "abs" => FunctionOperations::Absolute(Box::new(iter.next().unwrap())),
                    "cos" => FunctionOperations::Cos(Box::new(iter.next().unwrap())),
                    "sin" => FunctionOperations::Sin(Box::new(iter.next().unwrap())),
                    "tan" => FunctionOperations::Tan(Box::new(iter.next().unwrap())),
                    "ln" => FunctionOperations::Ln(Box::new(iter.next().unwrap())),
                    "log2" => FunctionOperations::Log2(Box::new(iter.next().unwrap())),
                    "sqrt" => FunctionOperations::SquareRoot(Box::new(iter.next().unwrap())),
                    //Multi arg functions
                    "mod" => FunctionOperations::Mod(Box::new(iter.next().unwrap()), Box::new(iter.next().unwrap())),
                    "smoothmin" => FunctionOperations::SmoothMin(Box::new(iter.next().unwrap()), Box::new(iter.next().unwrap()), Box::new(iter.next().unwrap())),
                    "smoothmax" => FunctionOperations::SmoothMax(Box::new(iter.next().unwrap()), Box::new(iter.next().unwrap()), Box::new(iter.next().unwrap())),
                    "mix" => FunctionOperations::Mix(Box::new(iter.next().unwrap()), Box::new(iter.next().unwrap()), Box::new(iter.next().unwrap())),
                    "clamp" => FunctionOperations::Clamp(Box::new(iter.next().unwrap()), Box::new(iter.next().unwrap()), Box::new(iter.next().unwrap())),
                    _=> unreachable!()
                    };

                    return Ok(Expression::Function(op))
                } else {
                    return Ok(Expression::Variable(name))
                }
            }
            _=> return Err(format!("Unexpected token on index: {}", self.pointer))
        }
    }
}