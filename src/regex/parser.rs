/// Contains regex parser.
use crate::regex::{Char, Regex};

use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    iter::Peekable,
    mem::size_of,
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
            // _ => write!(f, "Other error happenned"),
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
    pub fn parse(&'a mut self, re: &str) -> Result<(&'a mut Regex<'a>, usize), ParseError> {
        if self.in_use {
            return Err(ParseError::AlreadyInUse);
        }

        self.in_use = true;
        let mut chars = re.chars().peekable();
        let result = self.parse_expr_list(&mut chars)?;
        Ok((result, self.arena.len() * size_of::<Regex>()))
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
            if item.is_none() {
                break;
            }

            let peeked = chars.peek();
            match peeked {
                Some('+') => {
                    chars.next();
                    terms.push(self.arena.alloc(Regex::Repeat {
                        expr: item.ok_or(ParseError::ShouldEscape)?,
                        min: 1,
                        max: None,
                    }));
                }
                Some('*') => {
                    chars.next();
                    terms.push(self.arena.alloc(Regex::Repeat {
                        expr: item.ok_or(ParseError::ShouldEscape)?,
                        min: 0,
                        max: None,
                    }));
                }
                Some('?') => {
                    chars.next();
                    terms.push(self.arena.alloc(Regex::Repeat {
                        expr: item.ok_or(ParseError::ShouldEscape)?,
                        min: 0,
                        max: Some(1),
                    }));
                }
                Some('{') => {
                    chars.next();
                    let lower = parse_number(chars)?.unwrap_or(0);
                    match chars.peek().copied().ok_or(ParseError::UnexpectedEos)? {
                        ',' => {
                            chars.next();
                            let upper = parse_number(chars)?;
                            expect_char(chars, '}')?;
                            terms.push(self.arena.alloc(Regex::Repeat {
                                expr: item.ok_or(ParseError::ShouldEscape)?,
                                min: lower,
                                max: upper,
                            }));
                        }
                        '}' => {
                            chars.next();
                            terms.push(self.arena.alloc(Regex::Repeat {
                                expr: item.ok_or(ParseError::ShouldEscape)?,
                                min: lower,
                                max: Some(lower),
                            }));
                        }
                        _ => return Err(ParseError::ShouldEscape),
                    }
                }
                Some(_) => {
                    terms.push(item.ok_or(ParseError::ShouldEscape)?);
                }
                None => {
                    terms.push(item.ok_or(ParseError::ShouldEscape)?);
                    break;
                }
            }
        }
        Ok(Regex::sequence_from_iter(&self.arena, terms))
    }

    /// Parses `TERM`.
    fn parse_term(
        &'a self,
        chars: &mut Peekable<Chars>,
    ) -> Result<Option<&'a mut Regex<'a>>, ParseError> {
        match chars.peek() {
            Some('(') => {
                chars.next();
                let expr_list = self.parse_expr_list(chars)?;
                expect_char(chars, ')')?;
                Ok(Some(expr_list))
            }
            Some('[') => {
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
                Ok(Some(Regex::anyof_from_iter(&self.arena, charlist)))
            }
            _ => self.parse_char(chars),
        }
    }

    /// Parses `CHAR`.
    fn parse_char(
        &'a self,
        chars: &mut Peekable<Chars>,
    ) -> Result<Option<&'a mut Regex<'a>>, ParseError> {
        match chars.peek().copied() {
            None => Ok(None),
            Some('+' | '?' | '*' | '(' | ')' | '[' | ']' | '{' | '}' | '|') => Ok(None),
            Some('.') => {
                chars.next();
                Ok(Some(self.arena.alloc(Regex::Literal(Char::Any))))
            }
            Some('\\') => {
                chars.next();
                match chars.next().ok_or(ParseError::UnexpectedEos)? {
                    'd' => Ok(Some(self.arena.alloc(Regex::Literal(Char::Number)))),
                    'w' => Ok(Some(self.arena.alloc(Regex::Literal(Char::Alphabet)))),
                    c => Ok(Some(self.arena.alloc(Regex::Literal(Char::Just(c))))),
                }
            }
            Some(c) => {
                chars.next();
                Ok(Some(self.arena.alloc(Regex::Literal(Char::Just(c)))))
            }
        }
    }
}

/// Parses number.
fn parse_number(chars: &mut Peekable<Chars>) -> Result<Option<usize>, ParseError> {
    let mut number_str = String::with_capacity(16);
    loop {
        match chars.peek().copied() {
            Some(n @ '0'..='9') => {
                chars.next();
                number_str.push(n);
            }
            Some(_c @ (',' | '}')) => break,
            Some(c) => {
                return Err(ParseError::UnexpectedChar {
                    expected: '0',
                    actual: c,
                })
            }
            None => return Err(ParseError::UnexpectedEos),
        }
    }

    if number_str == "" {
        Ok(None)
    } else {
        Ok(Some(
            number_str.parse().map_err(|_| ParseError::ShouldEscape)?,
        ))
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
