mod regex;

use crate::regex::{Char, Regex};

use typed_arena::Arena;

fn main() {
    let arena = Arena::with_capacity(1024);

    // Expected: Equivalent to \w{5}you(0\d){2,}
    let regex = Regex::sequence_from_iter(
        &arena,
        vec![
            // \w{5}
            arena.alloc(Regex::Repeat {
                expr: arena.alloc(Regex::Literal(Char::Alphabet)),
                min: 5,
                max: Some(5),
            }),
            // y o u
            arena.alloc(Regex::Literal(Char::Just('y'))),
            arena.alloc(Regex::Literal(Char::Just('o'))),
            arena.alloc(Regex::Literal(Char::Just('u'))),
            // (0\d){2,}
            arena.alloc(Regex::Repeat {
                expr: Regex::sequence_from_iter(
                    &arena,
                    vec![
                        arena.alloc(Regex::Literal(Char::Just('0'))),
                        arena.alloc(Regex::Literal(Char::Number)),
                    ],
                ),
                min: 2,
                max: None,
            }),
        ],
    );
    println!("{}", regex);
}
