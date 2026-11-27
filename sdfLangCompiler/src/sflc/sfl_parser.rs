use super::expressions::{BinaryOperations, Expression, FunctionOperations};
use super::sfl_lexer::SdflTokens;
use super::wgsl_lowering::GenerateShader;
use crate::sflc::sfl_lexer::SdflTokens::Identifier;
use std::collections::{HashMap, HashSet};

pub struct Scene {
    pub args: Vec<String>,
    pub expressions: Vec<Expression>,
}

#[derive(Debug)]
pub struct Shape {
    pub args: Vec<String>,
    pub expression: Expression,
}
pub struct Context {
    pub consts: HashMap<String, f32>,
    pub shapes: HashMap<String, Shape>,
}
impl Context {
    pub fn new() -> Self {
        Self {
            consts: HashMap::new(),
            shapes: HashMap::new(),
        }
    }
}
pub struct SdflParser {
    pub tokens: Vec<SdflTokens>,
    pub pointer: usize,
}

//MATH PARSING INTO Expression
impl SdflParser {
    pub fn parse_math_expr(
        &mut self,
        end_tok: &SdflTokens,
        context: &Context,
        vars: Option<&HashSet<String>>,
    ) -> Result<Expression, String> {
        let expr = self.parse_additive(context, vars)?;

        if std::mem::discriminant(self.peek()) != std::mem::discriminant(end_tok) {
            return Err(format!(
                "Expected {:?} but found {:?} at index {}",
                end_tok,
                self.peek(),
                self.pointer
            ));
        }

        Ok(expr)
    }

    fn parse_additive(
        &mut self,
        context: &Context,
        vars: Option<&HashSet<String>>,
    ) -> Result<Expression, String> {
        let mut left = self.parse_multiplication(context, vars)?;
        while matches!(self.peek(), SdflTokens::MathPlus | SdflTokens::MathMinus) {
            let op = match self.advance() {
                SdflTokens::MathPlus => BinaryOperations::Add,
                SdflTokens::MathMinus => BinaryOperations::Subtract,
                _ => unreachable!(),
            };

            let right = self.parse_multiplication(context, vars)?;

            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplication(
        &mut self,
        context: &Context,
        vars: Option<&HashSet<String>>,
    ) -> Result<Expression, String> {
        let mut left = self.parse_power(context, vars)?;

        while matches!(self.peek(), SdflTokens::MathTimes | SdflTokens::MathDivide) {
            let op = match self.advance() {
                SdflTokens::MathTimes => BinaryOperations::Multiply,
                SdflTokens::MathDivide => BinaryOperations::Divide,
                _ => unreachable!(),
            };

            let right = self.parse_power(context, vars)?;

            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_power(
        &mut self,
        context: &Context,
        vars: Option<&HashSet<String>>,
    ) -> Result<Expression, String> {
        let mut left = self.parse_unary(context, vars)?;

        if matches!(self.peek(), SdflTokens::MathPow) {
            self.advance();
            let op = BinaryOperations::Power;

            let right = self.parse_power(context, vars)?;

            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    fn parse_unary(
        &mut self,
        context: &Context,
        vars: Option<&HashSet<String>>,
    ) -> Result<Expression, String> {
        if matches!(self.peek(), SdflTokens::MathMinus) {
            self.advance();

            let expr = self.parse_unary(context, vars)?;

            let op = FunctionOperations::Minus(Box::new(expr));

            return Ok(Expression::Function(op));
        }

        self.parse_primary(context, vars)
    }

    fn parse_primary(
        &mut self,
        context: &Context,
        vars: Option<&HashSet<String>>,
    ) -> Result<Expression, String> {
        match self.advance() {
            SdflTokens::Num(n) => Ok(Expression::Number(*n)),
            SdflTokens::OpenPar => {
                let expr = self.parse_additive(context, vars)?;

                if !matches!(self.peek(), SdflTokens::ClosedPar) {
                    return Err("Expected ')'".to_string());
                }

                self.advance();

                Ok(expr)
            }
            SdflTokens::Identifier(name) => {
                let name = name.clone();

                if matches!(self.peek(), SdflTokens::OpenPar) {
                    self.advance();

                    let expected_args = match name.as_str() {
                        "smoothmin" | "smoothmax" | "clamp" | "mix" => 3,
                        "mod" => 2,
                        "abs" | "cos" | "sin" | "tan" | "ln" | "log2" | "sqrt" => 1,
                        _ => return Err(format!("Unkown function: '{}'", name)),
                    };

                    let mut args: Vec<Expression> = Vec::with_capacity(expected_args);

                    if !matches!(self.peek(), SdflTokens::ClosedPar) {
                        args.push(self.parse_additive(context, vars)?);

                        while matches!(self.peek(), SdflTokens::Comma) {
                            self.advance();
                            args.push(self.parse_additive(context, vars)?);
                        }
                    }

                    if !matches!(self.peek(), SdflTokens::ClosedPar) {
                        return Err("Expected ')'".to_string());
                    }

                    self.advance();

                    if args.len() != expected_args {
                        return Err(format!(
                            "Function: {} expects {} args but got {}",
                            name,
                            expected_args,
                            args.len()
                        ));
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
                        "mod" => FunctionOperations::Mod(
                            Box::new(iter.next().unwrap()),
                            Box::new(iter.next().unwrap()),
                        ),
                        "smoothmin" => FunctionOperations::SmoothMin(
                            Box::new(iter.next().unwrap()),
                            Box::new(iter.next().unwrap()),
                            Box::new(iter.next().unwrap()),
                        ),
                        "smoothmax" => FunctionOperations::SmoothMax(
                            Box::new(iter.next().unwrap()),
                            Box::new(iter.next().unwrap()),
                            Box::new(iter.next().unwrap()),
                        ),
                        "mix" => FunctionOperations::Mix(
                            Box::new(iter.next().unwrap()),
                            Box::new(iter.next().unwrap()),
                            Box::new(iter.next().unwrap()),
                        ),
                        "clamp" => FunctionOperations::Clamp(
                            Box::new(iter.next().unwrap()),
                            Box::new(iter.next().unwrap()),
                            Box::new(iter.next().unwrap()),
                        ),
                        _ => unreachable!(),
                    };

                    Ok(Expression::Function(op))
                } else {
                    Ok(self.parse_identif(context, &name)?)
                }
            }
            _ => Err(format!("Unexpected token on index: {}", self.pointer)),
        }
    }
}
//Parser Utils
impl SdflParser {
    pub fn new(tokens: Vec<SdflTokens>) -> Self {
        Self { tokens, pointer: 0 }
    }

    pub fn peek(&self) -> &SdflTokens {
        self.tokens.get(self.pointer).unwrap_or(&SdflTokens::Eof)
    }

    //Constructors parsing
    pub fn advance(&mut self) -> &SdflTokens {
        let tok = self.tokens.get(self.pointer).unwrap_or(&SdflTokens::Eof);

        if *tok != SdflTokens::Eof {
            self.pointer += 1;
        }
        tok
    }

    pub fn wgsl_parse(&mut self) -> Result<String, String> {
        let context = Context::new();
        while self.advance() != &SdflTokens::Identifier("scene".to_string()) {
            if let Identifier(id) = self.advance() {
                match id.as_str() {
                    "const" => {
                        self.parse_const(&mut context);
                    }
                    "shape" => {
                        self.parse_shape_fn(&mut context);
                    }
                    _ => return Err(format!("Unkown Key: {}", id)),
                }
            }
        }
        let scene: Scene = self.parse_scene()?;
        Ok(scene.expressions.generate_shader(&scene.args)?)
    }
}

impl SdflTokens {
    pub fn expect(&self, expected: &SdflTokens) -> Result<(), String> {
        if self != expected {
            return Err(format!("Expected: `{:?}`, found: `{:?}`", expected, self));
        }

        Ok(())
    }
}
