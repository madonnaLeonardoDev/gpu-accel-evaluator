use std::collections::{HashMap, HashSet};
use wgpu::naga::common::wgsl;

use super::parser::{Expression, FunctionOperations, BinaryOperations, Parser};
use super::lexer::tokenize_expr;
const WGSL_CODE:&str = include_str!("./shader.wgsl");

fn format_wgsl_f32(n: f32) -> String {
    let clean = if n == 0.0 { 0.0 } else { n };
    if clean.fract() == 0.0 {
        format!("{clean:.1}")
    } else {
        format!("{clean}")
    }
}
pub trait GenerateShader{
    fn generate_shader(&self, vars_map:&HashMap<String, u32>) -> Result<String, String>;
}

impl GenerateShader for Vec<Expression> {
    fn generate_shader(&self, vars_map:&HashMap<String, u32>) -> Result<String, String> {
        if self.is_empty() {
            return Err("Empty Expression vec".to_string());
        }

        let mut wgsl_expr = String::new();

        if self.len() == 1 {
            wgsl_expr.push_str(&self[0].map_wgsl_math(vars_map)?);
        } else {
            for (i ,expr) in self.iter().enumerate() {
                let wgsl_e = expr.map_wgsl_math(vars_map)?;
                if i == self.len() - 1{
                    wgsl_expr.push_str(&wgsl_e);
                } else {
                    wgsl_expr.push_str(&format!("min({},",wgsl_e));
                }
                
            };
        }
        wgsl_expr.push_str(&")".repeat(self.len() - 1));

        let shader = WGSL_CODE.replace("//{WGSL EXPRESSION}", &wgsl_expr);

        Ok(shader)
    }
}


impl Expression {
    pub fn map_wgsl_math(&self, vars_map: &HashMap<String, u32>) -> Result<String, String> {
    match self {
        Expression::Number(n) => Ok(format_wgsl_f32(*n)),
        Expression::Variable(s) => match s.as_str() {
            "x" => Ok("p.x".to_string()),
            "y" => Ok("p.y".to_string()),
            "z" => Ok("p.z".to_string()),
            val => {
                let idx = vars_map
                    .get(val)
                    .ok_or_else(|| format!("No variable found called: {}", val))?;
                Ok(format!("vars[{}]", idx))
            }
        },
        Expression::Binary { left, operator, right } => {
            let l = left.map_wgsl_math(vars_map)?;
            let r = right.map_wgsl_math(vars_map)?;
            match operator {
                BinaryOperations::Add => Ok(format!("({} + {})", l, r)),
                BinaryOperations::Subtract => Ok(format!("({} - {})", l, r)),
                BinaryOperations::Multiply => Ok(format!("({} * {})", l, r)),
                BinaryOperations::Divide => Ok(format!("({} / {})", l, r)),
                BinaryOperations::Power => Ok(format!("pow({}, {})", l, r)),
            }
        }
        Expression::Function(f) => match f {
            FunctionOperations::Minus(a) => Ok(format!("(-({}))", a.map_wgsl_math(vars_map)?)),
            FunctionOperations::Absolute(a) => Ok(format!("abs({})", a.map_wgsl_math(vars_map)?)),
            FunctionOperations::Cos(a) => Ok(format!("cos({})", a.map_wgsl_math(vars_map)?)),
            FunctionOperations::Sin(a) => Ok(format!("sin({})", a.map_wgsl_math(vars_map)?)),
            FunctionOperations::Tan(a) => Ok(format!("tan({})", a.map_wgsl_math(vars_map)?)),
            FunctionOperations::Ln(a) => Ok(format!("log({})", a.map_wgsl_math(vars_map)?)),
            FunctionOperations::Log2(a) => Ok(format!("log2({})", a.map_wgsl_math(vars_map)?)),
            FunctionOperations::SquareRoot(a) => Ok(format!("sqrt({})", a.map_wgsl_math(vars_map)?)),
            FunctionOperations::Mod(a, b) => {
                Ok(format!("fmod({}, {})", a.map_wgsl_math(vars_map)?, b.map_wgsl_math(vars_map)?))
            }
            FunctionOperations::SmoothMin(a, b, c) => Ok(format!(
                "smin({}, {}, {})",
                a.map_wgsl_math(vars_map)?,
                b.map_wgsl_math(vars_map)?,
                c.map_wgsl_math(vars_map)?
            )),
            FunctionOperations::SmoothMax(a, b, c) => Ok(format!(
                "smax({}, {}, {})",
                a.map_wgsl_math(vars_map)?,
                b.map_wgsl_math(vars_map)?,
                c.map_wgsl_math(vars_map)?
            )),
            FunctionOperations::Mix(a, b, c) => Ok(format!(
                "mix({}, {}, {})",
                a.map_wgsl_math(vars_map)?,
                b.map_wgsl_math(vars_map)?,
                c.map_wgsl_math(vars_map)?
            )),
            FunctionOperations::Clamp(a, b, c) => Ok(format!(
                "clamp({}, {}, {})",
                a.map_wgsl_math(vars_map)?,
                b.map_wgsl_math(vars_map)?,
                c.map_wgsl_math(vars_map)?
            )),
        },
    }
}
}
pub trait IndexingHashMap {
    fn index_variables(&self) -> Result<Vec<f32>, String>;
}

impl IndexingHashMap for HashMap<String, u32> {
    fn index_variables(&self) -> Result<Vec<f32>, String> {
        if self.is_empty(){
            return Ok(Vec::new());
        }
        let mut seen: HashSet<u32> = HashSet::new();
        let mut i_sum: u64 = 0;
        for (_, &i) in self {
            if !seen.insert(i){
                return Err(format!("Duplicate index: {}", i));
            }
            i_sum += i as u64;
        }
        let len = self.len() as u64;
        if i_sum != len * (len - 1) /2 {
            return Err("Non compact indexes".to_string())
        }

        let vars_vec = vec![0f32; self.len()];

        Ok(vars_vec)
    }
}

pub trait CompileString {
    fn compile_wgsl(&self, vars_map: &HashMap<String, u32>) -> Result<String, String>;
}


impl CompileString for Vec<&str>{
    fn compile_wgsl(&self, vars_map: &HashMap<String, u32>) -> Result<String, String>{
        let mut exprs: Vec<Expression> = Vec::new();
        
        for str in self {
            let tokens = tokenize_expr(str)?;

            let expr = {
            let mut parser = Parser::new(&tokens);
            parser.parse_tokens()?
            };

            exprs.push(expr);
        }


        exprs.generate_shader(vars_map)
    }
}