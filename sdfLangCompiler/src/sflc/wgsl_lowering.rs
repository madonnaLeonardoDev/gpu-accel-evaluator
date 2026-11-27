use super::expressions::{BinaryOperations, Expression, FunctionOperations};
const WGSL_CODE: &str = include_str!("./shader.wgsl");

fn format_wgsl_f32(n: f32) -> String {
    let clean = if n == 0.0 { 0.0 } else { n };
    if clean.fract() == 0.0 {
        format!("{clean:.1}")
    } else {
        format!("{clean}")
    }
}
pub trait GenerateShader {
    fn generate_shader(&self, vars: &Vec<String>) -> Result<String, String>;
}

impl GenerateShader for Vec<Expression> {
    fn generate_shader(&self, vars: &Vec<String>) -> Result<String, String> {
        if self.is_empty() {
            return Err("Empty Expression vec".to_string());
        }

        let mut wgsl_expr = String::new();

        if self.len() == 1 {
            wgsl_expr.push_str(&self[0].map_wgsl_math(vars)?);
        } else {
            for (i, expr) in self.iter().enumerate() {
                let wgsl_e = expr.map_wgsl_math(vars)?;
                if i == self.len() - 1 {
                    wgsl_expr.push_str(&wgsl_e);
                } else {
                    wgsl_expr.push_str(&format!("min({},", wgsl_e));
                }
            }
        }
        wgsl_expr.push_str(&")".repeat(self.len() - 1));

        let shader = WGSL_CODE.replace("//{WGSL EXPRESSION}", &wgsl_expr);

        Ok(shader)
    }
}

impl Expression {
    pub fn map_wgsl_math(&self, vars: &Vec<String>) -> Result<String, String> {
        match self {
            Expression::Number(n) => Ok(format_wgsl_f32(*n)),
            Expression::Variable(s) => match s.as_str() {
                s if s == vars[0] => Ok("p.x".to_string()),
                s if s == vars[1] => Ok("p.y".to_string()),
                s if s == vars[2] => Ok("p.z".to_string()),
                _ => Err(format!("{} is not a valid argument", s)),
            },
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let l = left.map_wgsl_math(vars)?;
                let r = right.map_wgsl_math(vars)?;
                match operator {
                    BinaryOperations::Add => Ok(format!("({} + {})", l, r)),
                    BinaryOperations::Subtract => Ok(format!("({} - {})", l, r)),
                    BinaryOperations::Multiply => Ok(format!("({} * {})", l, r)),
                    BinaryOperations::Divide => Ok(format!("({} / {})", l, r)),
                    BinaryOperations::Power => Ok(format!("pow({}, {})", l, r)),
                }
            }
            Expression::Function(f) => match f {
                FunctionOperations::Minus(a) => Ok(format!("(-({}))", a.map_wgsl_math(vars)?)),
                FunctionOperations::Absolute(a) => Ok(format!("abs({})", a.map_wgsl_math(vars)?)),
                FunctionOperations::Cos(a) => Ok(format!("cos({})", a.map_wgsl_math(vars)?)),
                FunctionOperations::Sin(a) => Ok(format!("sin({})", a.map_wgsl_math(vars)?)),
                FunctionOperations::Tan(a) => Ok(format!("tan({})", a.map_wgsl_math(vars)?)),
                FunctionOperations::Ln(a) => Ok(format!("log({})", a.map_wgsl_math(vars)?)),
                FunctionOperations::Log2(a) => Ok(format!("log2({})", a.map_wgsl_math(vars)?)),
                FunctionOperations::SquareRoot(a) => {
                    Ok(format!("sqrt({})", a.map_wgsl_math(vars)?))
                }
                FunctionOperations::Mod(a, b) => Ok(format!(
                    "fmod({}, {})",
                    a.map_wgsl_math(vars)?,
                    b.map_wgsl_math(vars)?
                )),
                FunctionOperations::SmoothMin(a, b, c) => Ok(format!(
                    "smin({}, {}, {})",
                    a.map_wgsl_math(vars)?,
                    b.map_wgsl_math(vars)?,
                    c.map_wgsl_math(vars)?
                )),
                FunctionOperations::SmoothMax(a, b, c) => Ok(format!(
                    "smax({}, {}, {})",
                    a.map_wgsl_math(vars)?,
                    b.map_wgsl_math(vars)?,
                    c.map_wgsl_math(vars)?
                )),
                FunctionOperations::Mix(a, b, c) => Ok(format!(
                    "mix({}, {}, {})",
                    a.map_wgsl_math(vars)?,
                    b.map_wgsl_math(vars)?,
                    c.map_wgsl_math(vars)?
                )),
                FunctionOperations::Clamp(a, b, c) => Ok(format!(
                    "clamp({}, {}, {})",
                    a.map_wgsl_math(vars)?,
                    b.map_wgsl_math(vars)?,
                    c.map_wgsl_math(vars)?
                )),
            },
        }
    }
}
