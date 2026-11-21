mod window_rendering;
mod bytecode_constants;
mod parser;
mod lexer;
mod bytecode;
use std::collections::HashMap;
mod movement;

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

    let sphere = "sqrt((mod(x + 1.0, 2.0) - 1.0) * (mod(x + 1.0, 2.0) - 1.0) + y * y + (mod(z + 1.0, 2.0) - 1.0) * (mod(z + 1.0, 2.0) - 1.0)) - 0.5";


    let tokens: Vec<Tokens> = exec_res(tokenize(sphere));
    let bytecode = exec_res(tokens.compile(&vars_map));

    let scene = Scene{
        bytecode,
        camera: camera,
        vars: vars
    };

    run_window(&scene);
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
