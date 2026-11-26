use std::collections::HashMap;
mod window;
mod test;
mod shader_compiling;
mod sflc;

use crate::{sflc::sfl_lexer::tokenize_sfl, shader_compiling::wgsl_compile::CompileString};
use crate::test::formulas;
use window::window_rendering::{Scene, run_window};
use shader_compiling::wgsl_compile::{IndexingHashMap};
use sflc::sfl_parser::SdflParser;

fn main() {
    /*let mut camera_vars_map: HashMap<String, u32> = HashMap::new();
    camera_vars_map.insert("x".to_string(), 0);
    camera_vars_map.insert("y".to_string(), 1);
    camera_vars_map.insert("z".to_string(), 2);
    camera_vars_map.insert("yaw".to_string(), 3);
    camera_vars_map.insert("pitch".to_string(), 4);

    let mut vars_map: HashMap<String, u32> = HashMap::new();
    vars_map.insert("r".to_string(), 0);
    vars_map.insert("R".to_string(), 1);

    let mut camera = exec_res(camera_vars_map.index_variables());
    camera[0] = 0.0;
    camera[1] = 0.0;
    camera[2] = 0.0;
    camera[3] = 0.0;
    camera[4] = 0.0;
    let mut vars = exec_res(vars_map.index_variables());
    vars[0] = 0.0;
    vars[1] = 0.0;


    let test_shapes = vec!["smoothmax(sqrt((x + 3)^2 + (z + 3)^2) - 0.7, abs(y) - 1, 0.05)", "sqrt((x - 2)^2 + (y - 5)^2 + z^2) - 1", "mix(sqrt(x^2 + y^2 + z^2) - 1, smoothmax(smoothmax(abs(x) - 0.8, abs(y) - 0.8, 0.02), abs(z) - 0.8, 0.02), 0.5)"];
    
    let shader_src = exec_res(test_shapes.compile_wgsl(&vars_map));

    let scene = Scene { vars, camera };
    run_window(&scene, &shader_src); */
    println!("Start");
    let input = "carota = 2 + 1;";
    let tok = exec_res(tokenize_sfl(input));
    println!("{:?}", tok);
    let mut parser = SdflParser::new(tok);
    let res = exec_res(parser.parse_var());
    println!("{:?}", res);
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
