use std::io::{self, Write};

use crate::lexer::tokenize;

mod parser;
mod lexer;

fn main() {
    println!("Enter expression");

    io::stdout().flush().unwrap();

    let mut input = String::new();

    io::stdin().read_line(&mut input).expect("Failed reading input");

    let input = input.trim();

    let tokens = match tokenize(input) {
        Ok(t) => t,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    for token in tokens {
        println!("{:?}", token);
    }
}
