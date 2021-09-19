/// Contains regex parser.
use crate::regex::{Char, Regex};

use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    iter::Peekable,
    str::Chars,
    vec,
};

use typed_arena::Arena;

/// Represents an error of regex parser.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseError {
    /// The parser instance is already used.
    AlreadyInUse,

    UnexpectedChar {
        expected: char,
        actual: char,
    },

    /// Unexpected control char detected.
    ShouldEscape,

    /// Unexpected EOS detected.
    UnexpectedEos,

    /// Unknown other error.
    OtherError,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ParseError::AlreadyInUse => write!(f, "Parser already in use"),
            ParseError::UnexpectedChar { expected, actual } => {
                write!(f, "Unexpected char '{}', expected '{}'", actual, expected)
            }
            ParseError::ShouldEscape => write!(f, "Unexpected control char detected"),
            ParseError::UnexpectedEos => write!(f, "Unexpected EOS detected"),
            ParseError::OtherError => write!(f, "Other error happenned"),
        }
    }
}

impl Error for ParseError {}

/// Regex parser.
pub struct Parser<'a> {
    arena: Arena<Regex<'a>>,
    in_use: bool,
}

impl<'a> Parser<'a> {
    /// Creates new parser instance.
    pub fn new() -> Parser<'a> {
        let arena = Arena::new();
        Parser {
            arena,
            in_use: false,
        }
    }

    /// Parses a regex.
    pub fn parse(&'a mut self, re: &str) -> Result<&'a mut Regex<'a>, ParseError> {
        if self.in_use {
            return Err(ParseError::AlreadyInUse);
        }

        self.in_use = true;
        let mut chars = re.chars().peekable();
        self.parse_expr_list(&mut chars)
    }

    /// Parses `EXPRLIST`.
    fn parse_expr_list(
        &'a self,
        chars: &mut Peekable<Chars>,
    ) -> Result<&'a mut Regex<'a>, ParseError> {
        let mut seqs = vec![];
        loop {
            let item = self.parse_expr_seq(chars)?;
            seqs.push(item);

            let peeked = chars.peek();
            if peeked != Some(&'|') {
                break;
            }
            chars.next();
        }
        Ok(Regex::anyof_from_iter(&self.arena, seqs))
    }

    /// Parses `EXPRSEQ`.
    fn parse_expr_seq(
        &'a self,
        chars: &mut Peekable<Chars>,
    ) -> Result<&'a mut Regex<'a>, ParseError> {
        let mut terms = vec![];
        loop {
            let item = self.parse_term(chars)?;
            let peeked = chars.peek();
            match peeked {
                Some('+') => {
                    chars.next();
                    terms.push(self.arena.alloc(Regex::Repeat {
                        expr: item,
                        min: 1,
                        max: None,
                    }));
                }
                Some('*') => {
                    chars.next();
                    terms.push(self.arena.alloc(Regex::Repeat {
                        expr: item,
                        min: 0,
                        max: None,
                    }));
                }
                Some('?') => {
                    chars.next();
                    terms.push(self.arena.alloc(Regex::Repeat {
                        expr: item,
                        min: 0,
                        max: Some(1),
                    }));
                }
                Some(_) => {
                    terms.push(item);
                }
                None => break,
                // TODO: NUMSPEC に対応
            }
        }
        Ok(Regex::sequence_from_iter(&self.arena, terms))
    }

    /// Parses `EXPR`.
    fn parse_term(&'a self, chars: &mut Peekable<Chars>) -> Result<&'a mut Regex<'a>, ParseError> {
        match chars.peek().ok_or(ParseError::UnexpectedEos)? {
            '(' => {
                chars.next();
                let expr_list = self.parse_expr_list(chars)?;
                expect_char(chars, ')')?;
                Ok(expr_list)
            }
            '[' => {
                chars.next();
                let mut charlist = vec![];
                loop {
                    match self.parse_char(chars)? {
                        Some(c) => charlist.push(c),
                        None => {
                            expect_char(chars, ']')?;
                            break;
                        }
                    }
                }
                Ok(Regex::anyof_from_iter(&self.arena, charlist))
            }
            _ => self.parse_char(chars)?.ok_or(ParseError::ShouldEscape),
        }
    }

    /// Parses `CHAR`.
    fn parse_char(
        &'a self,
        chars: &mut Peekable<Chars>,
    ) -> Result<Option<&'a mut Regex<'a>>, ParseError> {
        match chars.peek() {
            None => Ok(None),
            Some('+' | '?' | '*' | '(' | ')' | '[' | ']' | '|') => Ok(None),
            Some('\\') => {
                chars.next();
                match chars.next().ok_or(ParseError::UnexpectedEos)? {
                    'd' => Ok(Some(self.arena.alloc(Regex::Literal(Char::Number)))),
                    'w' => Ok(Some(self.arena.alloc(Regex::Literal(Char::Alphabet)))),
                    c => Ok(Some(self.arena.alloc(Regex::Literal(Char::Just(c))))),
                }
            }
            Some(c) => Ok(Some(self.arena.alloc(Regex::Literal(Char::Just(*c))))),
        }
    }
}

/// Expects specific char on stream.
fn expect_char(chars: &mut Peekable<Chars>, c: char) -> Result<(), ParseError> {
    let peeked = *chars.peek().ok_or(ParseError::UnexpectedEos)?;
    if peeked == c {
        chars.next();
        Ok(())
    } else {
        Err(ParseError::UnexpectedChar {
            expected: c,
            actual: peeked,
        })
    }
}
