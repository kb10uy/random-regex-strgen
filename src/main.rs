mod regex;

use crate::regex::{ParseError, Parser};

use std::env::args;

fn main() -> Result<(), ParseError> {
    let args: Vec<String> = args().skip(1).collect();

    for regex_str in args {
        println!("Input  : {}", regex_str);
        
        let mut parser = Parser::new();
        match parser.parse(&regex_str) {
            Ok(regex) => println!("Result : {}", regex),
            Err(err) => println!("Error  : {}", err),
        }

        println!();
    }

    Ok(())
}
