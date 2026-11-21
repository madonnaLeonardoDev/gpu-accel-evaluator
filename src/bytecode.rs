use std::collections::{HashMap, HashSet};
use crate::parser::{Expression, FunctionOperations, BinaryOperations};
use crate::bytecode_constants::*;
pub enum OpCodes{
    Num(f32),
    Var(u32),

    //1 Argument Operations
    Minus,
    Absolute,
    Cos,
    Sin,
    Tan,
    Ln,
    Log2,
    SquareRoot,

    //2 Args Operations
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Mod,

    //3 Args Operations
    SmoothMin,
    SmoothMax,
    Mix,
    Clamp
}
impl OpCodes {
    fn push_byte(&self, out: &mut Vec<u32>) {
        match self {
            //2 bytecode
            Self::Num(n) => {
                out.push(NUM);
                out.push(f32::to_bits(*n))
            }
            Self::Var(v) => {
                out.push(VAR);
                out.push(*v);
            }
            //1 bytecode
            Self::Minus => out.push(MINUS),
            Self::Absolute => out.push(ABSOLUTE),
            Self::Cos => out.push(COS),
            Self::Sin => out.push(SIN),
            Self::Tan => out.push(TAN),
            Self::Ln => out.push(LN),
            Self::Log2 => out.push(LOG2),
            Self::SquareRoot => out.push(SQUAREROOT),
            Self::Add => out.push(ADD),
            Self::Subtract => out.push(SUBTRACT),
            Self::Multiply => out.push(MULTIPLY),
            Self::Divide => out.push(DIVIDE),
            Self::Power => out.push(POWER),
            Self::Mod => out.push(MOD),
            Self::SmoothMin => out.push(SMOOTHMIN),
            Self::SmoothMax => out.push(SMOOTHMAX),
            Self::Mix => out.push(MIX),
            Self::Clamp =>out.push(CLAMP)
        }
    }
}

    fn format_wgsl_f32(n: f32) -> String {
        // Handle -0.0 cleanly and force a decimal point for whole numbers
        let clean = if n == 0.0 { 0.0 } else { n };
        if clean.fract() == 0.0 {
            format!("{clean:.1}") // e.g. 2.0 -> "2.0", -1.0 -> "-1.0"
        } else {
            format!("{clean}")
        }
    }

impl Expression {
    fn node_count(&self) -> usize {
        match self {
            Expression::Number(_) | Expression::Variable(_) => 1,
            
            Expression::Binary { left, operator: _ , right } => 1 + left.node_count() + right.node_count(),

            Expression::Function(op) => {
                match op {
                    FunctionOperations::Minus(a) |
                    FunctionOperations::Absolute(a) |
                    FunctionOperations::Cos(a) |
                    FunctionOperations::Sin(a) |
                    FunctionOperations::Tan(a) |
                    FunctionOperations::Ln(a) |
                    FunctionOperations::Log2(a) |
                    FunctionOperations::SquareRoot(a) => 1 + a.node_count(),

                    FunctionOperations::Mod(a, b) => 1 + a.node_count() + b.node_count(),

                    FunctionOperations::SmoothMin(a, b ,c) |
                    FunctionOperations::SmoothMax(a, b ,c) |
                    FunctionOperations::Mix(a, b ,c) |
                    FunctionOperations::Clamp(a, b ,c) => 1 + a.node_count() + b.node_count() + c.node_count()
                }
            } 
        }
    }
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


    fn push_opcode(self, variables: &HashMap<String, u32>, out: &mut Vec<OpCodes>) -> Result<(), String>{
        match self {
            Self::Number(n) => Ok(out.push(OpCodes::Num(n))),
            Self::Variable(v) => {
                let idx = match variables.get(&v){
                    None => return  Err(format!("No variable matching: {}",v)),
                    Some(v) => v
                };
                Ok(out.push(OpCodes::Var(*idx)))
            },
            Self::Binary { left, operator, right } => {
                left.push_opcode(variables, out)?;
                right.push_opcode(variables, out)?;
                let opcode = match operator {
                    BinaryOperations::Add => OpCodes::Add,
                    BinaryOperations::Subtract => OpCodes::Subtract,
                    BinaryOperations::Multiply => OpCodes::Multiply,
                    BinaryOperations::Divide => OpCodes::Divide,
                    BinaryOperations::Power => OpCodes::Power
                };
                out.push(opcode);
                Ok(())
            },
            Self::Function(f) => {
                let opcode = match f {
                        FunctionOperations::Minus(a) => {
                            a.push_opcode(variables, out)?;
                            OpCodes::Minus
                        }
                        FunctionOperations::Absolute(a) => {
                            a.push_opcode(variables, out)?;
                            OpCodes::Absolute
                        }
                        FunctionOperations::Cos(a) => {
                            a.push_opcode(variables, out)?;
                            OpCodes::Cos
                        }
                        FunctionOperations::Sin(a) => {
                            a.push_opcode(variables, out)?;
                            OpCodes::Sin
                        }
                        FunctionOperations::Tan(a) => {
                            a.push_opcode(variables, out)?;
                            OpCodes::Tan
                        }
                        FunctionOperations::Ln(a) => {
                            a.push_opcode(variables, out)?;
                            OpCodes::Ln
                        }
                        FunctionOperations::Log2(a) => {
                            a.push_opcode(variables, out)?;
                            OpCodes::Log2
                        }
                        FunctionOperations::SquareRoot(a) => {
                            a.push_opcode(variables, out)?;
                            OpCodes::SquareRoot
                        }
                        FunctionOperations::Mod(a, b) => {
                            a.push_opcode(variables, out)?;
                            b.push_opcode(variables, out)?;
                            OpCodes::Mod
                        },
                        FunctionOperations::SmoothMin(a, b ,c) => {
                            a.push_opcode(variables, out)?;
                            b.push_opcode(variables, out)?;
                            c.push_opcode(variables, out)?;
                            OpCodes::SmoothMin
                        }
                        FunctionOperations::SmoothMax(a, b ,c) => {
                            a.push_opcode(variables, out)?;
                            b.push_opcode(variables, out)?;
                            c.push_opcode(variables, out)?;
                            OpCodes::SmoothMax
                        }
                        FunctionOperations::Mix(a, b ,c) => {
                            a.push_opcode(variables, out)?;
                            b.push_opcode(variables, out)?;
                            c.push_opcode(variables, out)?;
                            OpCodes::Mix
                        }
                        FunctionOperations::Clamp(a, b ,c) => {
                            a.push_opcode(variables, out)?;
                            b.push_opcode(variables, out)?;
                            c.push_opcode(variables, out)?;
                            OpCodes::Clamp
                        }
                };

                out.push(opcode);

                Ok(())
            }
        }
    }

    pub fn flatten(self, variables: &HashMap<String, u32>) -> Result<Vec<OpCodes>, String> {
        let mut out: Vec<OpCodes> = Vec::with_capacity(self.node_count());

        self.push_opcode(variables, &mut out)?;

        Ok(out)
    }
}

pub trait ExecuteInstructions {
    fn into_bytecode(&self) -> Vec<u32>;
}

impl ExecuteInstructions for Vec<OpCodes> {
    fn into_bytecode(&self) -> Vec<u32> {
        let mut res = Vec::with_capacity(self.len() + self.len() / 2);
        for op in self {
            op.push_byte(&mut res);
        }
        res
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

