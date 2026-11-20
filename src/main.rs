use crate::tests::{base_test, trig_tests};

mod gpu_eval;
mod tests;
mod bytecode_constants;
mod parser;
mod lexer;
mod bytecode;

fn main() {
//RUNNING TESTS
match base_test(){
    Ok(r) => println!("Base Test {:?}",r),
    Err(e) => println!("{}", e)
}
match trig_tests(){
    Ok(r) => println!("Trig Test {:?}",r),
    Err(e) => println!("{}", e)
}
}
