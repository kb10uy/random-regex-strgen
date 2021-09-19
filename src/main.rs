mod regex;

use crate::regex::{ParseError, Parser};

fn main() -> Result<(), ParseError> {
    let mut parser = Parser::new();
    let regex = parser.parse(r#"\w{5}you(0\d)+"#)?;

    // Expected: Equivalent to \w{5}you(0\d){2,}
    println!("{}", regex);

    Ok(())
}
