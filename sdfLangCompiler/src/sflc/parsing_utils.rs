use super::sfl_lexer::SdflTokens::{self, *};
use super::sfl_parser::{SdflParser, Shape};
use crate::sflc::expressions::Expression;
use crate::sflc::expressions::FunctionOperations;
use crate::sflc::sfl_parser::{Context, Scene};
use std::collections::{HashMap, HashSet};
impl SdflParser {
    pub fn parse_shape_fn(&mut self, context: &mut Context) -> Result<(), String> {
        let id = match &self.peek() {
            Identifier(id) => {
                if context.shapes.contains_key(id) {
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
        let expr = self.parse_math_expr(&SdflTokens::ClosedCurly, context, Some(&vars_map))?;
        self.advance().expect(&ClosedCurly)?;

        context.shapes.insert(
            id,
            Shape {
                args: vars,
                expression: expr,
            },
        );

        Ok(())
    }

    pub fn parse_const(&mut self, context: &mut Context) -> Result<(), String> {
        let id = match self.peek() {
            Identifier(id) => {
                if context.consts.contains_key(id) {
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
        let expr = self.parse_math_expr(&SdflTokens::SemiColon, context, None)?;
        let res = expr.resolve_compiletime(context, self)?;

        context.consts.insert(id, res);
        Ok(())
    }

    fn parse_shape_to_expr(
        &mut self,
        context: &Context,
        shape: &Shape,
        vars: Option<&HashSet<String>>,
    ) -> Result<Expression, String> {
        self.advance().expect(&OpenPar)?;
        let mut shape_args: HashMap<String, Expression> = HashMap::new();
        for (i, arg_name) in shape.args.iter().enumerate() {
            if i > 0 {
                self.advance().expect(&Comma)?;
            }
            let end_tok = if i == shape.args.len() - 1 {
                ClosedPar
            } else {
                Comma
            };
            let expr_res = self.parse_math_expr(&end_tok, context, vars)?;
            shape_args.insert(arg_name.clone(), expr_res);
        }
        let mut to_res_expr = shape.expression.clone();
        to_res_expr.insert_expressions(&shape_args)?;
        self.advance();
        Ok(to_res_expr)
    }

    pub fn parse_identif(
        &mut self,
        context: &Context,
        vars: Option<&HashSet<String>>,
        id: &str,
    ) -> Result<Expression, String> {
        if let Some(num) = context.consts.get(id) {
            return Ok(Expression::Number(*num));
        }
        if let Some(shape) = context.shapes.get(id) {
            let res = self.parse_shape_to_expr(context, shape, vars)?;
            return Ok(res);
        }
        if let Some(v) = vars
            && v.contains(id)
        {
            return Ok(Expression::Variable(id.to_string()));
        }
        Err(format!(
            "Identifier {} doesnt match any previous statements",
            id
        ))
    }
    pub fn parse_scene(&mut self, context: &mut Context) -> Result<Scene, String> {
        if let Identifier(id) = self.peek()
            && id == "scene"
        {
            self.advance();
            self.advance().expect(&OpenPar)?;
            let mut args: Vec<String> = Vec::new();
            let mut vars_map: HashSet<String> = HashSet::new();
            let mut i: usize = 0;
            while !matches!(self.peek(), &ClosedPar) {
                if i > 0 {
                    self.advance().expect(&Comma)?;
                }
                if let Identifier(id) = self.advance() {
                    if !vars_map.insert(id.to_string()) {
                        return Err(format!("Duplicate argument declared in shape: {}", id));
                    }
                    args.push(id.clone());
                } else {
                    return Err(format!("Expected Identifier found: {:?}", self.peek()));
                }
                i += 1;
            }
            let mut exprs: Vec<Expression> = Vec::new();
            self.advance().expect(&ClosedPar)?;
            self.advance().expect(&OpenBracket)?;
            while !matches!(self.peek(), ClosedBracket) {
                let expr = self.parse_math_expr(&SemiColon, context, Some(&vars_map))?;
                exprs.push(expr);
                self.advance().expect(&SemiColon)?;
            }
            return Ok(Scene {
                expressions: exprs,
                args,
            });
        }

        Err(format!("Expected Identifier found: {:?}", self.peek()))
    }
}

impl Expression {
    fn replace(&mut self, new_expr: Expression) {
        *self = new_expr;
    }
    pub fn insert_expressions(
        &mut self,
        inserts: &HashMap<String, Expression>,
    ) -> Result<(), String> {
        match self {
            Self::Variable(id) => match inserts.get(id) {
                None => return Err(format!("No value matches {}", id)),
                Some(expr) => {
                    self.replace(expr.clone());
                }
            },
            Self::Binary {
                left,
                operator: _,
                right,
            } => {
                left.insert_expressions(inserts)?;
                right.insert_expressions(inserts)?;
            }
            Self::Function(f) => match f {
                FunctionOperations::Minus(a)
                | FunctionOperations::Absolute(a)
                | FunctionOperations::Cos(a)
                | FunctionOperations::Sin(a)
                | FunctionOperations::Tan(a)
                | FunctionOperations::Ln(a)
                | FunctionOperations::Log2(a)
                | FunctionOperations::SquareRoot(a) => a.insert_expressions(inserts)?,
                FunctionOperations::Mod(a, b) => {
                    a.insert_expressions(inserts)?;
                    b.insert_expressions(inserts)?;
                }
                FunctionOperations::SmoothMin(a, b, c)
                | FunctionOperations::SmoothMax(a, b, c)
                | FunctionOperations::Mix(a, b, c)
                | FunctionOperations::Clamp(a, b, c) => {
                    a.insert_expressions(inserts)?;
                    b.insert_expressions(inserts)?;
                    c.insert_expressions(inserts)?;
                }
            },

            _ => {}
        }

        Ok(())
    }
}
