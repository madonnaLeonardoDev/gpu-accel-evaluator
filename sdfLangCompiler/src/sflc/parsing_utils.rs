use crate::sflc::expressions::Expression;

use super::sfl_lexer::SdflTokens::{self, *};
use super::sfl_parser::{SdflParser, Shape};
use std::collections::HashSet;
impl SdflParser {
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

    pub fn resolve_shape(&mut self) -> Result<Expression, String> {
        match self.peek() {
            Identifier(id) => {}
            _ => Err(format!("Expected Identifier found {:?}", self.peek())),
        }
    }
}
