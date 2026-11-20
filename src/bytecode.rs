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

pub trait ExecuteBytecode {
    fn cpu_execute(&self, vars: &[f32]) -> f32;
}
impl ExecuteBytecode for Vec<u32> {
    fn cpu_execute(&self, vars: &[f32]) -> f32 {
        let mut i: usize = 0;
        let mut stack: Vec<f32> = Vec::with_capacity(32);

        while i < self.len() {
            match self[i] {
                NUM => {
                    i+=1;
                    stack.push(f32::from_bits(self[i]));
                }
                VAR => {
                    i+=1;
                    stack.push(vars[self[i] as usize]);
                }
                MINUS => {
                    let num= stack.pop().expect("Expected f32");
                    stack.push(num * -1.0);
                }
                ABSOLUTE => {
                    let num = stack.pop().expect("Expected f32");
                    stack.push(num.abs());
                }
                COS => {
                    let num = stack.pop().expect("Expected f32");
                    stack.push(num.cos());
                }
                SIN => {
                    let num = stack.pop().expect("Expected f32");
                    stack.push(num.sin());
                }
                TAN => {
                    let num = stack.pop().expect("Expected f32");
                    stack.push(num.tan());
                }
                LN => {
                    let num = stack.pop().expect("Expected f32");
                    stack.push(num.ln());
                }
                LOG2 => {
                    let num = stack.pop().expect("Expected f32");
                    stack.push(num.log2());
                }
                SQUAREROOT => {
                    let num = stack.pop().expect("Expected f32");
                    stack.push(num.sqrt());
                }
                ADD => {
                    let right = stack.pop().expect("Expected f32");
                    let left = stack.pop().expect("Expected f32");
                    stack.push(left + right);
                }
                SUBTRACT => {
                    let right = stack.pop().expect("Expected f32");
                    let left = stack.pop().expect("Expected f32");
                    stack.push(left - right);
                }
                MULTIPLY => {
                    let right = stack.pop().expect("Expected f32");
                    let left = stack.pop().expect("Expected f32");
                    stack.push(left * right);
                }
                DIVIDE => {
                    let right = stack.pop().expect("Expected f32");
                    let left = stack.pop().expect("Expected f32");
                    stack.push(left / right);
                }
                POWER => {
                    let right = stack.pop().expect("Expected f32");
                    let left = stack.pop().expect("Expected f32");
                    stack.push(left.powf(right));
                }
                MOD => {
                    let right = stack.pop().expect("Expected f32");
                    let left = stack.pop().expect("Expected f32");
                    stack.push(left % right);
                }
                SMOOTHMIN=> {
                    let k = stack.pop().expect("Expected f32");
                    let b = stack.pop().expect("Expected f32");
                    let a = stack.pop().expect("Expected f32");
                    let h = ((k - (a - b).abs()).max(0.0)) / k;
                    stack.push(a.min(b) - h * h * k * 0.25);
                }
                SMOOTHMAX => {
                    let k = stack.pop().expect("Expected f32");
                    let b = stack.pop().expect("Expected f32");
                    let a = stack.pop().expect("Expected f32");
                    let h = ((k - (a - b).abs()).max(0.0)) / k;
                    stack.push(a.max(b) + h * h * k * 0.25);
                }
                MIX => {
                    let t = stack.pop().expect("Expected f32");
                    let y = stack.pop().expect("Expected f32");
                    let x = stack.pop().expect("Expected f32");
                    stack.push(x + (y - x) * t);
                }
                CLAMP => {
                    let max_val = stack.pop().expect("Expected f32");
                    let min_val = stack.pop().expect("Expected f32");
                    let x = stack.pop().expect("Expected f32");
                    stack.push(x.clamp(min_val, max_val));
                }
                _=> panic!()
            }
            i += 1;
        }


        *stack.last().expect("Expected result on stack")
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