use std::collections::{HashMap, HashSet};
use crate::parser::{Expression, FunctionOperations, BinaryOperations};

enum OpCodes{
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
    Log,
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
                    FunctionOperations::Log(a) |
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

    fn get_variables(&self, seen_vars: &mut HashSet<String>, out:&mut Vec<String>) {
        match self {
            Expression::Variable(str) => {
                if !seen_vars.insert(str.clone()){
                    out.push(str.clone());
                }
            },
            Expression::Binary { left, operator: _, right } => {
                left.get_variables(seen_vars, out);
                right.get_variables(seen_vars, out);
            },
            Expression::Function(function) => {
                    match function {
                        FunctionOperations::Minus(a) |
                        FunctionOperations::Absolute(a) |
                        FunctionOperations::Cos(a) |
                        FunctionOperations::Sin(a) |
                        FunctionOperations::Tan(a) |
                        FunctionOperations::Ln(a) |
                        FunctionOperations::Log2(a) |
                        FunctionOperations::Log(a) |
                        FunctionOperations::SquareRoot(a) => a.get_variables(seen_vars, out),

                        FunctionOperations::Mod(a, b) => {
                        a.get_variables(seen_vars, out);
                        b.get_variables(seen_vars, out);
                        },
                        FunctionOperations::SmoothMin(a, b ,c) |
                        FunctionOperations::SmoothMax(a, b ,c) |
                        FunctionOperations::Mix(a, b ,c) |
                        FunctionOperations::Clamp(a, b ,c) => {
                        a.get_variables(seen_vars, out);
                        b.get_variables(seen_vars, out);
                        c.get_variables(seen_vars, out);
                        }
                }
            }
            Expression::Number(_) => {}
        }
    }

    fn vars(&self) -> Vec<String> {
        let mut seen_vars: HashSet<String> = HashSet::new();
        let mut vars: Vec<String> = Vec::new();
        self.get_variables(&mut seen_vars, &mut vars);

        vars
    }

    pub fn into_instructions(self, variables: &HashMap<String, u32>) {

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
                        FunctionOperations::Log(a) => {
                            a.push_opcode(variables, out)?;
                            OpCodes::Log
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

    pub fn into_bytecode(self, variables: &HashMap<String, u32>) -> Result<Vec<OpCodes>, String> {
        let mut out: Vec<OpCodes> = Vec::with_capacity(self.node_count());

        self.push_opcode(variables, &mut out)?;

        Ok(out)
    }

}

trait ExecuteInstructions {
    fn execute(&self, variables_vec: &[f32]) -> Result<f32, String>;
}

impl ExecuteInstructions for Vec<OpCodes<u32>> {
    fn execute(&self, variables_vec: &[f32]) -> Result<f32, String>{
        
    }
}

trait IndexingHashMap {
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