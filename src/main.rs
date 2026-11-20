use crate::tests::full_tests;

mod tests;
mod bytecode_constants;
mod parser;
mod lexer;
mod bytecode;

fn main() {
//RUNNING TESTS
match full_tests(){
    Ok(r) => println!("{:?}",r),
    Err(e) => println!("{}", e)
}
}
