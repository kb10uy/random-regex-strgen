mod rand_gen;
mod regex;

use crate::{
    rand_gen::RandomGenerator,
    regex::{ParseError, Parser},
};

use std::env::args;

use rand::prelude::*;

fn main() -> Result<(), ParseError> {
    let args: Vec<String> = args().skip(1).collect();
    let mut randgen = RandomGenerator::new(thread_rng(), 20);

    for regex_str in args {
        println!("Input  : {}", regex_str);

        let mut parser = Parser::new();
        match parser.parse(&regex_str) {
            Ok((regex, size)) => {
                println!("Result : {}", regex);
                println!("AST    : {:?}", regex);
                println!("Usage  : {}bytes", size);
                println!("Random generation");
                for _ in 0..20 {
                    let string = randgen.generate(regex).expect("Unexpected error");
                    println!("{}", string);
                }
            }
            Err(err) => println!("Error  : {}", err),
        }

        println!();
    }

    Ok(())
}
