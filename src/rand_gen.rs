use crate::regex::{Char, Regex};

use std::io::{prelude::*, Result as IoResult};

use rand::{prelude::*, seq::SliceRandom};

const RANDOM_CHARS: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

pub struct RandomGenerator<R> {
    rng: R,
    quantity_upper: usize,
}

impl<R: Rng> RandomGenerator<R> {
    pub fn new(rng: R, quantity_upper: usize) -> RandomGenerator<R> {
        RandomGenerator {
            rng,
            quantity_upper,
        }
    }

    pub fn generate(&mut self, regex: &Regex<'_>) -> IoResult<String> {
        let mut buffer = vec![];
        self.write_regex(&mut buffer, regex)?;
        Ok(String::from_utf8(buffer).expect("Should contain only UTF-8"))
    }

    fn write_regex<W: Write>(&mut self, writer: &mut W, regex: &Regex<'_>) -> IoResult<()> {
        match regex {
            Regex::Tail => (),
            Regex::Literal(c) => match c {
                Char::Just(c) => write!(writer, "{}", c)?,
                Char::Alphabet => {
                    let index = self.rng.gen_range(0..26);
                    write!(writer, "{}", &RANDOM_CHARS[index..(index + 1)])?;
                }
                Char::Number => {
                    let index = self.rng.gen_range(26..36);
                    write!(writer, "{}", &RANDOM_CHARS[index..(index + 1)])?;
                }
                Char::Any => {
                    let index = self.rng.gen_range(0..36);
                    write!(writer, "{}", &RANDOM_CHARS[index..(index + 1)])?;
                }
            },
            Regex::Sequence { .. } => {
                for item in regex.iter().expect("Should have items") {
                    self.write_regex(writer, item)?;
                }
            }
            Regex::AnyOf { .. } => {
                let items: Vec<_> = regex
                    .iter()
                    .expect("Should have items")
                    .map(|r| (r, r.random_weight()))
                    .collect();
                let (item, _) = items
                    .choose_weighted(&mut self.rng, |x| x.1)
                    .expect("Should have at least one item");
                self.write_regex(writer, item)?;
            }
            Regex::Repeat { expr, min, max } => {
                let upper = max.unwrap_or(self.quantity_upper);
                let range = *min..=upper;
                for _ in 0..(self.rng.gen_range(range)) {
                    self.write_regex(writer, expr)?;
                }
            }
        }
        Ok(())
    }
}
