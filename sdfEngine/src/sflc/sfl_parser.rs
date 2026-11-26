use crate::sflc::sfl_lexer::SdflTokens::{self, *};
use crate::shader_compiling::parser::Expression::Variable;
use crate::shader_compiling::parser::{BinaryOperations, Expression, FunctionOperations};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Shape {
    pub args: Vec<String>,
    pub expression: Expression,
}

pub struct SdflParser {
    tokens: Vec<SdflTokens>,
    pointer: usize,
    shapes: HashMap<String, Shape>,
    consts: HashMap<String, f32>,
}

//MATH PARSING INTO Expression
impl SdflParser {
    pub fn parse_math_expr(
        &mut self,
        end_tok: &SdflTokens,
        vars: Option<&HashSet<String>>,
    ) -> Result<Expression, String> {
        let expr = self.parse_additive(vars)?;

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

    fn parse_additive(&mut self, vars: Option<&HashSet<String>>) -> Result<Expression, String> {
        let mut left = self.parse_multiplication(vars)?;
        while matches!(self.peek(), SdflTokens::MathPlus | SdflTokens::MathMinus) {
            let op = match self.advance() {
                SdflTokens::MathPlus => BinaryOperations::Add,
                SdflTokens::MathMinus => BinaryOperations::Subtract,
                _ => unreachable!(),
            };

            let right = self.parse_multiplication(vars)?;

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
        vars: Option<&HashSet<String>>,
    ) -> Result<Expression, String> {
        let mut left = self.parse_power(vars)?;

        while matches!(self.peek(), SdflTokens::MathTimes | SdflTokens::MathDivide) {
            let op = match self.advance() {
                SdflTokens::MathTimes => BinaryOperations::Multiply,
                SdflTokens::MathDivide => BinaryOperations::Divide,
                _ => unreachable!(),
            };

            let right = self.parse_power(vars)?;

            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_power(&mut self, vars: Option<&HashSet<String>>) -> Result<Expression, String> {
        let mut left = self.parse_unary(vars)?;

        if matches!(self.peek(), SdflTokens::MathPow) {
            self.advance();
            let op = BinaryOperations::Power;

            let right = self.parse_power(vars)?;

            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    fn parse_unary(&mut self, vars: Option<&HashSet<String>>) -> Result<Expression, String> {
        if matches!(self.peek(), SdflTokens::MathMinus) {
            self.advance();

            let expr = self.parse_unary(vars)?;

            let op = FunctionOperations::Minus(Box::new(expr));

            return Ok(Expression::Function(op));
        }

        self.parse_primary(vars)
    }

    fn parse_primary(&mut self, vars: Option<&HashSet<String>>) -> Result<Expression, String> {
        match self.advance() {
            SdflTokens::Num(n) => return Ok(Expression::Number(*n)),
            SdflTokens::OpenPar => {
                let expr = self.parse_additive(vars)?;

                if !matches!(self.peek(), SdflTokens::ClosedPar) {
                    return Err("Expected ')'".to_string());
                }

                self.advance();

                return Ok(expr);
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
                        args.push(self.parse_additive(vars)?);

                        while matches!(self.peek(), SdflTokens::Comma) {
                            self.advance();
                            args.push(self.parse_additive(vars)?);
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

                    return Ok(Expression::Function(op));
                } else {
                    if let Some(var) = self.consts.get(&name) {
                        return Ok(Expression::Number(*var));
                    }
                    if let Some(shape) = self.shapes.get(&name) {
                        //TODO: Calculate shapes
                        todo!()
                    }
                    if vars.contains(&name) {
                        return Ok(Variable(name));
                    }
                    return Err(format!("{} Doesn't match any shapes or constant", name));
                }
            }
            _ => return Err(format!("Unexpected token on index: {}", self.pointer)),
        }
    }
}

impl SdflParser {
    pub fn new(tokens: Vec<SdflTokens>) -> Self {
        Self {
            tokens,
            pointer: 0,
            shapes: HashMap::new(),
            consts: HashMap::new(),
        }
    }

    fn peek(&self) -> &SdflTokens {
        self.tokens.get(self.pointer).unwrap_or(&SdflTokens::Eof)
    }

    fn advance(&mut self) -> &SdflTokens {
        let tok = self.tokens.get(self.pointer).unwrap_or(&SdflTokens::Eof);

        if *tok != SdflTokens::Eof {
            self.pointer += 1;
        }
        tok
    }

    pub fn parse_shape_fn(&mut self) -> Result<(String, Shape), String> {
        let id = match &self.peek() {
            Identifier(id) => {
                if self.shapes.contains_key(id) {
                    return Err(format!(
                        "Invalid Shape Identifier: {}, there already exists a Shape with this identifier",
                        id
                    ));
                }
                id.clone()
            }
            _ => {
                return Err(format!(
                    "Expected Shape Identifier found: {:?}",
                    self.peek()
                ));
            }
        };
        self.advance();
        let mut vars_map: HashSet<String> = HashSet::new();
        let mut vars: Vec<String> = Vec::new();
        self.advance().expect(&OpenPar)?;

        let mut i: usize = 0;
        while !matches!(self.peek(), &ClosedPar) {
            if i > 0 {
                self.advance().expect(&Comma)?;
            }
            if let Identifier(id) = self.advance() {
                if !vars_map.insert(id.to_string()) {
                    return Err(format!("Duplicate argument declared in shape: {}", id));
                }
                vars.push(id.to_string());
            } else {
                return Err(format!("Expected Identifier found: {:?}", self.peek()));
            }
            i += 1;
        }
        self.advance().expect(&ClosedPar)?;
        self.advance().expect(&OpenCurly)?;
        let expr = self.parse_math_expr(&SdflTokens::ClosedCurly, Some(&vars_map))?;
        self.advance().expect(&ClosedCurly)?;

        Ok((
            id,
            Shape {
                args: vars,
                expression: expr,
            },
        ))
    }

    pub fn parse_var(&mut self) -> Result<(String, f32), String> {
        let id = match self.peek() {
            Identifier(id) => {
                if self.consts.contains_key(id) {
                    return Err(format!(
                        "Invalid var Identifier: {}, there already exists a var with this identifier",
                        id
                    ));
                }
                id.clone()
            }
            _ => return Err(format!("Expected var Identifier found: {:?}", self.peek())),
        };
        self.advance();
        self.advance().expect(&SdflTokens::Equals)?;
        let expr = self.parse_math_expr(&SdflTokens::SemiColon, None)?;
        let res = expr.resolve_compiletime(&self)?;

        Ok((id, res))
    }
}

impl SdflTokens {
    fn expect(&self, expected: &SdflTokens) -> Result<(), String> {
        if self != expected {
            return Err(format!("Expected: `{:?}`, found: `{:?}`", expected, self));
        }

        Ok(())
    }
}

impl Expression {
    fn resolve_compiletime(&self, parser: &SdflParser) -> Result<f32, String> {
        match self {
            Self::Number(n) => Ok(*n),
            Self::Variable(id) => {
                if let Some(n) = parser.consts.get(id) {
                    return Ok(*n);
                }
                if let Some(shape) = parser.shapes.get(id) {
                    //TODO: Reslove Shape
                    return //Placeholder
                    Ok(3.0);
                } else {
                    return Err(format!("{} is not declared", id));
                }
            }
            Self::Binary {
                left,
                operator,
                right,
            } => match operator {
                BinaryOperations::Add => {
                    let left = left.resolve_compiletime(parser)?;
                    let right = right.resolve_compiletime(parser)?;
                    return Ok(left + right);
                }
                BinaryOperations::Subtract => {
                    let left = left.resolve_compiletime(parser)?;
                    let right = right.resolve_compiletime(parser)?;
                    return Ok(left - right);
                }
                BinaryOperations::Multiply => {
                    let left = left.resolve_compiletime(parser)?;
                    let right = right.resolve_compiletime(parser)?;
                    return Ok(left * right);
                }
                BinaryOperations::Divide => {
                    let left = left.resolve_compiletime(parser)?;
                    let right = right.resolve_compiletime(parser)?;
                    return Ok(left / right);
                }
                BinaryOperations::Power => {
                    let left = left.resolve_compiletime(parser)?;
                    let right = right.resolve_compiletime(parser)?;
                    return Ok(left.powf(right));
                }
            },
            Self::Function(f) => match f {
                FunctionOperations::Minus(a) => Ok(a.resolve_compiletime(parser)? * -1.0),
                FunctionOperations::Absolute(a) => Ok(a.resolve_compiletime(parser)?.abs()),
                FunctionOperations::Cos(a) => Ok(a.resolve_compiletime(parser)?.cos()),
                FunctionOperations::Sin(a) => Ok(a.resolve_compiletime(parser)?.sin()),
                FunctionOperations::Tan(a) => Ok(a.resolve_compiletime(parser)?.tan()),
                FunctionOperations::Ln(a) => Ok(a.resolve_compiletime(parser)?.ln()),
                FunctionOperations::Log2(a) => Ok(a.resolve_compiletime(parser)?.log2()),
                FunctionOperations::SquareRoot(a) => Ok(a.resolve_compiletime(parser)?.sqrt()),
                FunctionOperations::Mod(a, b) => Ok(sdf_mod(
                    a.resolve_compiletime(parser)?,
                    b.resolve_compiletime(parser)?,
                )),
                FunctionOperations::SmoothMin(a, b, c) => Ok(smoothmin(
                    a.resolve_compiletime(parser)?,
                    b.resolve_compiletime(parser)?,
                    c.resolve_compiletime(parser)?,
                )),
                FunctionOperations::SmoothMax(a, b, c) => Ok(smoothmax(
                    a.resolve_compiletime(parser)?,
                    b.resolve_compiletime(parser)?,
                    c.resolve_compiletime(parser)?,
                )),
                FunctionOperations::Mix(a, b, c) => Ok(mix(
                    a.resolve_compiletime(parser)?,
                    b.resolve_compiletime(parser)?,
                    c.resolve_compiletime(parser)?,
                )),
                FunctionOperations::Clamp(a, b, c) => Ok(a.resolve_compiletime(parser)?.clamp(
                    b.resolve_compiletime(parser)?,
                    c.resolve_compiletime(parser)?,
                )),
            },
        }
    }
}

fn sdf_mod(a: f32, b: f32) -> f32 {
    a - b * (a / b).floor()
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
    // or: a + (b - a) * t
}

fn smoothmin(a: f32, b: f32, k: f32) -> f32 {
    if k <= 0.0 {
        return a.min(b);
    }
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    mix(b, a, h) - k * h * (1.0 - h)
}

fn smoothmax(a: f32, b: f32, k: f32) -> f32 {
    if k <= 0.0 {
        return a.max(b);
    }
    let h = (0.5 - 0.5 * (b - a) / k).clamp(0.0, 1.0);
    mix(b, a, h) + k * h * (1.0 - h)
}
//Expression Parsing Impl
