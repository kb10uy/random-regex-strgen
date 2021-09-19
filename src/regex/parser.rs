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

    /// Unexpected EOS detected.
    UnexpectedEos,

    /// Unknown other error.
    OtherError,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ParseError::AlreadyInUse => write!(f, "Parser already in use"),
            ParseError::UnexpectedEos => write!(f, "Unexpected EOS detected"),
            ParseError::OtherError => write!(f, "Other error happenned"),
        }
    }
}

impl Error for ParseError {}

/// Represents the start of some groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseState {
    /// `[`
    StartAnyChar,

    /// `(`
    StartGroup,
}

/// Regex parser.
pub struct Parser<'a> {
    arena: Arena<Regex<'a>>,
    in_use: bool,
    parse_stack: Vec<ParseState>,
    ast_stack: Vec<&'a mut Regex<'a>>,
}

impl<'a> Parser<'a> {
    /// Creates new parser instance.
    pub fn new() -> Parser<'a> {
        let arena = Arena::new();
        Parser {
            arena,
            in_use: false,
            parse_stack: vec![],
            ast_stack: vec![],
        }
    }

    /// Creates new parser instance with arena capacity.
    pub fn with_capacity(cap: usize) -> Parser<'a> {
        let arena = Arena::with_capacity(cap);
        Parser {
            arena,
            in_use: false,
            parse_stack: vec![],
            ast_stack: vec![],
        }
    }

    /// Parses a regex.
    pub fn parse(&'a mut self, re: &str) -> Result<&'a mut Regex<'a>, ParseError> {
        if self.in_use {
            return Err(ParseError::AlreadyInUse);
        }

        let mut chars = re.chars().peekable();
        let mut escaping = false;
        while let Some(c) = chars.next() {
            match c {
                '(' if !escaping => {
                    self.parse_stack.push(ParseState::StartGroup);
                }
                ')' if !escaping => {}
                '[' if !escaping => {
                    self.parse_stack.push(ParseState::StartAnyChar);
                }
                ']' if !escaping => {}
                '\\' if !escaping => {
                    escaping = true;
                }
                _ => {
                    escaping = false;
                }
            }
        }

        todo!();
    }

    /// Parses `EXPR`.
    fn parse_expr(
        &'a mut self,
        chars: &mut Peekable<Chars>,
    ) -> Result<&'a mut Regex<'a>, ParseError> {
        match chars.peek() {
            None => Ok(self.arena.alloc(Regex::Tail)),
            Some('(') => {
                chars.next();
                todo!();
            }
            Some('[') => {
                chars.next();
                todo!();
            }
            Some(c) => {
                todo!();
            }
            _ => Err(ParseError::OtherError),
        }
    }

    /// Parses `CHAR`.
    fn parse_char(
        &'a mut self,
        chars: &mut Peekable<Chars>,
    ) -> Result<&'a mut Regex<'a>, ParseError> {
        match chars.next().ok_or(ParseError::UnexpectedEos)? {
            '\\' => match chars.next().ok_or(ParseError::UnexpectedEos)? {
                'd' => Ok(self.arena.alloc(Regex::Literal(Char::Number))),
                'w' => Ok(self.arena.alloc(Regex::Literal(Char::Alphabet))),
                c => Ok(self.arena.alloc(Regex::Literal(Char::Just(c)))),
            },
            c => Ok(self.arena.alloc(Regex::Literal(Char::Just(c)))),
        }
    }
}
