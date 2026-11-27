use crate::sflc::{expressions::Expression::Number, sfl_parser::Context};

use super::sfl_parser::SdflParser;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum BinaryOperations {
    Add,
    Subtract,
    Multiply,
    Divide,
    //`left` = Base | `right` = index -- sqrt(base, index)
    Power,
    //`left` = Base | `right` = exponent -- pow(base, exponent)
}

#[derive(Debug, Clone)]
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
impl Expression {
    pub fn resolve_compiletime(
        &self,
        context: &Context,
        parser: &SdflParser,
    ) -> Result<f32, String> {
        match self {
            Self::Number(n) => Ok(*n),
            Self::Variable(id) => {
                if let Some(n) = context.consts.get(id) {
                    return Ok(*n);
                }
                Err(format!("{} is not declared", id))
            }
            Self::Binary {
                left,
                operator,
                right,
            } => match operator {
                BinaryOperations::Add => {
                    let left = left.resolve_compiletime(context, parser)?;
                    let right = right.resolve_compiletime(context, parser)?;
                    Ok(left + right)
                }
                BinaryOperations::Subtract => {
                    let left = left.resolve_compiletime(context, parser)?;
                    let right = right.resolve_compiletime(context, parser)?;
                    Ok(left - right)
                }
                BinaryOperations::Multiply => {
                    let left = left.resolve_compiletime(context, parser)?;
                    let right = right.resolve_compiletime(context, parser)?;
                    Ok(left * right)
                }
                BinaryOperations::Divide => {
                    let left = left.resolve_compiletime(context, parser)?;
                    let right = right.resolve_compiletime(context, parser)?;
                    Ok(left / right)
                }
                BinaryOperations::Power => {
                    let left = left.resolve_compiletime(context, parser)?;
                    let right = right.resolve_compiletime(context, parser)?;
                    Ok(left.powf(right))
                }
            },
            Self::Function(f) => match f {
                FunctionOperations::Minus(a) => Ok(-(a.resolve_compiletime(context, parser)?)),
                FunctionOperations::Absolute(a) => {
                    Ok(a.resolve_compiletime(context, parser)?.abs())
                }
                FunctionOperations::Cos(a) => Ok(a.resolve_compiletime(context, parser)?.cos()),
                FunctionOperations::Sin(a) => Ok(a.resolve_compiletime(context, parser)?.sin()),
                FunctionOperations::Tan(a) => Ok(a.resolve_compiletime(context, parser)?.tan()),
                FunctionOperations::Ln(a) => Ok(a.resolve_compiletime(context, parser)?.ln()),
                FunctionOperations::Log2(a) => Ok(a.resolve_compiletime(context, parser)?.log2()),
                FunctionOperations::SquareRoot(a) => {
                    Ok(a.resolve_compiletime(context, parser)?.sqrt())
                }
                FunctionOperations::Mod(a, b) => Ok(sdf_mod(
                    a.resolve_compiletime(context, parser)?,
                    b.resolve_compiletime(context, parser)?,
                )),
                FunctionOperations::SmoothMin(a, b, c) => Ok(smoothmin(
                    a.resolve_compiletime(context, parser)?,
                    b.resolve_compiletime(context, parser)?,
                    c.resolve_compiletime(context, parser)?,
                )),
                FunctionOperations::SmoothMax(a, b, c) => Ok(smoothmax(
                    a.resolve_compiletime(context, parser)?,
                    b.resolve_compiletime(context, parser)?,
                    c.resolve_compiletime(context, parser)?,
                )),
                FunctionOperations::Mix(a, b, c) => Ok(mix(
                    a.resolve_compiletime(context, parser)?,
                    b.resolve_compiletime(context, parser)?,
                    c.resolve_compiletime(context, parser)?,
                )),
                FunctionOperations::Clamp(a, b, c) => {
                    Ok(a.resolve_compiletime(context, parser)?.clamp(
                        b.resolve_compiletime(context, parser)?,
                        c.resolve_compiletime(context, parser)?,
                    ))
                }
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
