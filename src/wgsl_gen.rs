use std::collections::HashMap;

use crate::parser::Expression;

const WGSL_CODE:&str = include_str!("./shader.wgsl");

pub fn generate_shader(expr: &Expression, vars_map: &HashMap<String, u32>) -> Result<String, String> {
    let wgsl_math = expr.map_wgsl_math(vars_map)?;
    let eval_sdf_fn = format!(
    "fn eval_sdf(p: vec3<f32>) -> f32 {{\n    return {};\n}}", wgsl_math
);
let final_wgsl = WGSL_CODE.replace("// {{EVAL_SDF}}", &eval_sdf_fn);

Ok(final_wgsl)
}