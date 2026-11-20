use std::collections::HashMap;
use crate::bytecode::{ExecuteBytecode, ExecuteInstructions, IndexingHashMap};
use crate::lexer::{tokenize};
use crate::parser::Parser;


fn test_str(str: &str,vars_map: &HashMap<String, u32>, vars: &Vec<f32> , expected: f32) -> Result<(bool, f32), String> {
    let tokens = tokenize(str)?;
    let mut parser = Parser::new(&tokens);
    let expr = parser.parse_tokens()?;
    let opcode = expr.flatten(&vars_map)?;
    let bytecode = opcode.into_bytecode();
    let res = bytecode.cpu_execute(&vars);

    if res == expected {
        return Ok((true , res))
    } else {
        return Ok((false, res));
    }
}

pub fn full_tests() -> Result<bool, String> {
    let mut vars_map: HashMap<String, u32> = HashMap::new();
    vars_map.insert("px".to_string(), 0);
    vars_map.insert("py".to_string(), 1);
    vars_map.insert("pz".to_string(), 2);
    vars_map.insert("k".to_string(), 3);
    vars_map.insert("a".to_string(),4);
    vars_map.insert("b".to_string(), 5);
    vars_map.insert("c".to_string(), 6);

    let mut vars = vars_map.index_variables()?;
        vars[0] = 21.4;    // vars[0] = px
        vars[1] = 1.023;   // vars[1] = py
        vars[2] = 23.0;    // vars[2] = pz
        vars[3] = 764.9;    // vars[3] = k
        vars[4] = 921.2;    // vars[4] = a
        vars[5] = 745.1023; // vars[5] = b
        vars[6] = 12.0;  // vars[6] = c


    let tests: Vec<(&str, f32)> = vec![
    ("1.0 + 2.0 * 3.0", 1.0 + 2.0 * 3.0),
    ("(1.0 + 2.0) * 3.0", (1.0 + 2.0) * 3.0),
    ("-px + abs(-py)", -vars[0] + (-vars[1]).abs()),
    ("cos(a) * sin(b) + tan(c * 0.5)", vars[4].cos() * vars[5].sin() + (vars[6] * 0.5).tan()),
    ("sqrt(px * px + py * py)", (vars[0] * vars[0] + vars[1] * vars[1]).sqrt()),
    ("ln(abs(a) + 1.0) + log2(abs(b) + 1.0)", (vars[4].abs() + 1.0).ln() + (vars[5].abs() + 1.0).log2()),
    ("a - b", vars[4] - vars[5]),
    ("b - a", vars[5] - vars[4]),
    ("a / (b + 1.0)", vars[4] / (vars[5] + 1.0)),
    ("px ^ py", vars[0].powf(vars[1])),
    ("mod(px, c)", vars[0] % vars[6]),
    ("mod(px + c * 0.5, c) - c * 0.5", ((vars[0] + vars[6] * 0.5) % vars[6]) - vars[6] * 0.5),
    ("mod(a - b, c)", (vars[4] - vars[5]) % vars[6]),
    ("smoothmin(px, py, k)", {
        let h = ((vars[3] - (vars[0] - vars[1]).abs()).max(0.0)) / vars[3];
        vars[0].min(vars[1]) - h * h * vars[3] * 0.25
    }),
    ("smoothmax(a, b, k)", {
        let h = ((vars[3] - (vars[4] - vars[5]).abs()).max(0.0)) / vars[3];
        vars[4].max(vars[5]) + h * h * vars[3] * 0.25
    }),
    ("mix(a, b, c)", vars[4] + (vars[5] - vars[4]) * vars[6]),
    ("clamp(px, -c, c)", vars[0].clamp(-vars[6], vars[6])),
    ("sqrt(px * px + py * py + pz * pz) - c", (vars[0] * vars[0] + vars[1] * vars[1] + vars[2] * vars[2]).sqrt() - vars[6]),
    ("smoothmin(sqrt(px * px + py * py + pz * pz) - b, abs(py) - c, k)", {
        let s1 = (vars[0] * vars[0] + vars[1] * vars[1] + vars[2] * vars[2]).sqrt() - vars[5];
        let s2 = vars[1].abs() - vars[6];
        let h = ((vars[3] - (s1 - s2).abs()).max(0.0)) / vars[3];
        s1.min(s2) - h * h * vars[3] * 0.25
    }),
    ("sin(mod(px, a) * b + sin(py * b)) * k + (sqrt(px * px + py * py) - c)", ((vars[0] % vars[4]) * vars[5] + (vars[1] * vars[5]).sin()).sin() * vars[3] + ((vars[0] * vars[0] + vars[1] * vars[1]).sqrt() - vars[6])),
    ("clamp(mix(a - k, b + k, clamp(pz * c + 0.5, 0.0, 1.0)), -a, a)", {
        let t = (vars[2] * vars[6] + 0.5).clamp(0.0, 1.0);
        let m = (vars[4] - vars[3]) + ((vars[5] + vars[3]) - (vars[4] - vars[3])) * t;
        m.clamp(-vars[4], vars[4])
    }),
];
    let mut is_passed = true;
    let mut fails: Vec<usize> = Vec::new();
    for (i, test) in tests.iter().enumerate(){
        let tr = test_str(test.0, &vars_map, &vars, test.1)?;
        println!("{:?}", tr);
        if !tr.0 {
            is_passed = false;
            fails.push(i);
        }
    }
    Ok(is_passed)
}