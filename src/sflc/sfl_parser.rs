use std::{collections::{HashSet}};
use crate::shader_compiling::parser::{Expression, BinaryOperations, FunctionOperations};
use crate::{sflc::sfl_lexer::SflTokens::{self, *}};

#[derive(Debug)]
pub struct Shape{
    pub variables: HashSet<String>,
    pub expression: Expression
}

pub struct SflParser{
    tokens: Vec<SflTokens>,
    pointer: usize
}
//MATH PARSING IMPL
impl SflParser {
    pub fn parse_math_expr(&mut self) -> Result<Expression, String> {
        let expr = self.parse_additive()?;

        if !matches!(self.peek(), SflTokens::Eof){
            return Err(format!("Unexpected token remaining at index {}", self.pointer))
        }

       Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_multiplication()?;

        while matches!(self.peek(), SflTokens::MathPlus | SflTokens::MathMinus ) {
            let op = match self.advance() {
                SflTokens::MathPlus => BinaryOperations::Add,
                SflTokens::MathMinus => BinaryOperations::Subtract,
                _ => unreachable!()
            };

            let right = self.parse_multiplication()?;

            left = Expression::Binary { left: Box::new(left), operator: op, right: Box::new(right) };
        }

        Ok(left)
    } 

    fn parse_multiplication(&mut self)-> Result<Expression, String> {
        let mut left = self.parse_power()?;

        while matches!(self.peek(), SflTokens::MathTimes | SflTokens::MathDivide) {
            let op = match self.advance() {
                SflTokens::MathTimes => BinaryOperations::Multiply,
                SflTokens::MathDivide => BinaryOperations::Divide,
                _=> unreachable!()
            };

            let right = self.parse_power()?;

            left = Expression::Binary { left: Box::new(left), operator: op, right: Box::new(right) };
        }

        Ok(left)
    }

    fn parse_power(&mut self)-> Result<Expression, String> {
        let mut left = self.parse_unary()?;

        while matches!(self.peek(), SflTokens::MathPow){
            self.advance();
            let op = BinaryOperations::Power;

            let right = self.parse_power()?;

            left = Expression::Binary { left: Box::new(left), operator: op, right: Box::new(right) }
        }

        Ok(left)
    }

    fn parse_unary(&mut self)-> Result<Expression, String> {
        if matches!(self.peek(), SflTokens::MathMinus){
            self.advance();

            let expr = self.parse_unary()?;

            let op = FunctionOperations::Minus(Box::new(expr));

            return Ok(Expression::Function(op))
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        match self.advance() {
            SflTokens::Num(n) => {
                return Ok(Expression::Number(*n))
            },
            SflTokens::OpenPar => {
                let expr = self.parse_additive()?;

                if !matches!(self.peek(), SflTokens::ClosedPar) {
                    return Err("Expected ')'".to_string())
                }

                self.advance();

                return Ok(expr)
            },
            SflTokens::Identifier(name ) => {
                let name = name.clone();

                if matches!(self.peek(), SflTokens::OpenPar) {
                    self.advance();
                    
                    let expected_args = match name.as_str() {
                        "smoothmin" | "smoothmax" | "clamp" | "mix" => 3,
                        "mod" => 2,
                        "abs" | "cos" | "sin" | "tan" | "ln" | "log2" | "sqrt" => 1,
                        _=> return Err(format!("Unkown function: '{}'", name))
                    };

                    let mut args: Vec<Expression> = Vec::with_capacity(expected_args);

                    if !matches!(self.peek(), SflTokens::ClosedPar) {
                        args.push(self.parse_additive()?);

                        while matches!(self.peek(), SflTokens::Comma){
                            self.advance();
                            args.push(self.parse_additive()?);
                        }
                    }

                    if !matches!(self.peek(), SflTokens::ClosedPar) {
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

impl SflParser {
    pub fn new(tokens: Vec<SflTokens>) -> Self {
        Self {
            tokens,
            pointer: 0
        }
    }

    fn peek(&mut self) -> &SflTokens {
        self.tokens.get(self.pointer).unwrap_or(&SflTokens::Eof)
    }

    fn advance(&mut self) -> &SflTokens {
        let tok = self.tokens.get(self.pointer).unwrap_or(&SflTokens::Eof);
        
        if *tok != SflTokens::Eof {
            self.pointer += 1;
        }
        tok
    }

    pub fn parse_shape(&mut self) -> Result<Shape, String> {
        let mut vars: HashSet<String> = HashSet::new();
        self.advance().expect(&OpenPar)?;

        let mut i: usize = 0;
        while !matches!(self.peek(),&ClosedPar) {
            if i > 0 {
                self.advance().expect(&Comma)?;
            }
            if let Identifier(id) = self.advance(){
                if !vars.insert(id.to_string()) {
                    return Err(format!("Duplicate argument declared in shape: {}", id))
                }
            }
            i+=1;
        }
        self.advance().expect(&ClosedPar)?;
        self.advance().expect(&OpenCurly)?;
        let expr = self.parse_math_expr()?;
        self.advance();
        
        
        Ok(Shape { variables: vars, expression: expr })
    }
}

impl SflTokens {
    fn expect(&self, expected: &SflTokens) -> Result<(), String> {
        if self != expected {
            return Err(format!("Expected: `{:?}`, found: `{:?}`",expected, self))
        }

        Ok(())
    }
}

//Expression Parsing Impl
