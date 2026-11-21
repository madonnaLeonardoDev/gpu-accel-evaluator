mod window_rendering;
mod bytecode_constants;
mod parser;
mod lexer;
mod bytecode;
use std::collections::HashMap;
mod movement;
mod wgsl_gen;
mod test;


use crate::parser::Parser;
use crate::test::formulas;
use crate::wgsl_gen::generate_shader;
use crate::window_rendering::{Scene, run_window};
use crate::bytecode::{IndexingHashMap};
use crate::lexer::{CompileTokens, tokenize, Tokens};

fn main() {
    let mut camera_vars_map: HashMap<String, u32> = HashMap::new();
    camera_vars_map.insert("x".to_string(), 0);
    camera_vars_map.insert("y".to_string(), 1);
    camera_vars_map.insert("z".to_string(), 2);
    camera_vars_map.insert("yaw".to_string(), 3);
    camera_vars_map.insert("pitch".to_string(), 4);

    let mut vars_map: HashMap<String, u32> = HashMap::new();
    vars_map.insert("x".to_string(), 0);
    vars_map.insert("y".to_string(), 1);
    vars_map.insert("z".to_string(), 2);
    vars_map.insert("r".to_string(), 3);
    vars_map.insert("R".to_string(), 4);

    let mut camera = exec_res(camera_vars_map.index_variables());
    camera[0] = 0.0;
    camera[1] = 0.0;
    camera[2] = 0.0;
    camera[3] = 0.0;
    camera[4] = 0.0;
    let mut vars = exec_res(vars_map.index_variables());
    vars[0] = 0.0;
    vars[1] = 0.0;
    vars[2] = 0.0;
    vars[3] = 0.0;
    vars[4] = 0.0;

    let test_shapes = formulas()[14];
    
    let expr = {
        let tokens = exec_res(tokenize(test_shapes));
        let mut parser = Parser::new(&tokens);
        exec_res(parser.parse_tokens())
    };
    let shader_src = exec_res(generate_shader(&expr, &vars_map));

    let scene = Scene{
        camera: camera,
        vars: vars,
    };

    run_window(&scene, &shader_src);
}
fn exec_res<R>(res: Result<R, String>) -> R {
    match res {
        Ok(res) =>return  res,
        Err(e) => {
            println!("{}", e);
            std::process::exit(1)
        }

    }
}
