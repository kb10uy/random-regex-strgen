/// Contains AST types for regex.

use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    iter::FusedIterator,
};

use typed_arena::Arena;

/// Represents a literal character in regex.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Char {
    /// Specific character
    Just(char),

    /// small alphabets `[a-z]`
    Alphabet,

    /// numbers `[0-9]`
    Number,
}

impl Char {
    /// Returns the weight of this `Char` instance for random generation.
    pub fn random_weight(&self) -> usize {
        match self {
            Char::Just(c) => 1,
            Char::Alphabet => 26,
            Char::Number => 10,
        }
    }
}

impl Display for Char {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Char::Just(c) => write!(f, "{}", c),
            Char::Alphabet => write!(f, "\\w"),
            Char::Number => write!(f, "\\d"),
        }
    }
}

/// Represents a node of regex.
#[derive(Debug)]
pub enum Regex<'a> {
    /// `ε`
    Tail,

    /// `X`
    Literal(Char),

    /// `XYZ`
    Sequence {
        head: &'a Regex<'a>,
        rest: &'a Regex<'a>,
    },

    /// `[XYZ]` or `(X|Y|Z)`
    AnyOf {
        head: &'a Regex<'a>,
        rest: &'a Regex<'a>,
    },

    /// `X?`, `X*`, `X+`, or `X{n,m}`
    Repeat {
        expr: &'a Regex<'a>,
        min: usize,
        max: Option<usize>,
    },
}

impl<'a> Regex<'a> {
    /// Constructs `Regex::Sequence` list from iterator.
    pub fn sequence_from_iter(
        arena: &'a Arena<Regex<'a>>,
        iter: impl IntoIterator<Item = &'a mut Regex<'a>>,
    ) -> &'a Regex<'a> {
        let mut iter: Vec<_> = iter.into_iter().collect();
        iter.reverse();
        match iter.len() {
            0 => arena.alloc(Regex::Tail),
            1 => iter.into_iter().next().expect("Should have just one item"),
            _ => {
                let mut rest = arena.alloc(Regex::Tail);
                for head in iter {
                    rest = arena.alloc(Regex::Sequence { head, rest });
                }
                rest
            }
        }
    }

    /// Constructs `Regex::AnyOf` list from iterator.
    pub fn anyof_from_iter(
        arena: &'a Arena<Regex<'a>>,
        iter: impl IntoIterator<Item = &'a mut Regex<'a>>,
    ) -> &'a Regex<'a> {
        let mut iter: Vec<_> = iter.into_iter().collect();
        iter.reverse();
        match iter.len() {
            0 => arena.alloc(Regex::Tail),
            1 => iter.into_iter().next().expect("Should have just one item"),
            _ => {
                let mut rest = arena.alloc(Regex::Tail);
                for head in iter {
                    rest = arena.alloc(Regex::AnyOf { head, rest });
                }
                rest
            }
        }
    }

    /// For `Sequence` and `AnyOf`, creates iterator for its elements.
    pub fn iter(&'a self) -> Option<Iter<'a>> {
        match self {
            Regex::Sequence { .. } | Regex::AnyOf { .. } => Some(Iter(&self)),
            _ => None,
        }
    }

    /// Judges whether this instance consists only of literals.
    pub fn has_only_literals(&self) -> bool {
        match self {
            Regex::Tail => true,
            Regex::Literal(_) => true,
            Regex::Sequence { head, rest } => match head {
                Regex::Literal(_) => rest.has_only_literals(),
                _ => false,
            },
            Regex::AnyOf { head, rest } => match head {
                Regex::Literal(_) => rest.has_only_literals(),
                _ => false,
            },
            Regex::Repeat { .. } => false,
        }
    }
}

/// Iterator for listed regex.
pub struct Iter<'a>(&'a Regex<'a>);

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Regex<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Regex::Tail => None,
            Regex::Sequence { head, rest } => {
                self.0 = rest;
                Some(head)
            }
            Regex::AnyOf { head, rest } => {
                self.0 = rest;
                Some(head)
            }
            _ => unreachable!("Invalid iterator state: {:?}", self.0),
        }
    }
}

impl<'a> FusedIterator for Iter<'a> {}

impl<'a> Display for Regex<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Regex::Tail => Ok(()),
            Regex::Literal(c) => c.fmt(f),
            Regex::Sequence { .. } => {
                let items = self.iter().expect("Should have iterator");
                for c in items {
                    c.fmt(f)?;
                }
                Ok(())
            }
            Regex::AnyOf { .. } => {
                if self.has_only_literals() {
                    write!(f, "[")?;
                    let items = self.iter().expect("Should have iterator");
                    for c in items {
                        c.fmt(f)?;
                    }
                    write!(f, "]")?;
                    Ok(())
                } else {
                    let items: Vec<_> = self.iter().expect("Should have iterator").collect();
                    if items.len() == 1 {
                        items[0].fmt(f)?;
                    } else {
                        items[0].fmt(f)?;
                        for item in &items[1..] {
                            write!(f, "|")?;
                            item.fmt(f)?;
                        }
                    }
                    Ok(())
                }
            }
            Regex::Repeat { expr, min, max } => {
                match expr {
                    Regex::Tail => write!(f, "()")?,
                    Regex::Literal(c) => c.fmt(f)?,
                    Regex::Sequence { .. } => {
                        write!(f, "(")?;
                        expr.fmt(f)?;
                        write!(f, ")")?;
                    }
                    Regex::AnyOf { .. } => {
                        if expr.has_only_literals() {
                            expr.fmt(f)?;
                        } else {
                            write!(f, "(")?;
                            expr.fmt(f)?;
                            write!(f, ")")?;
                        }
                    }
                    Regex::Repeat { .. } => {
                        expr.fmt(f)?;
                    }
                }
                match (min, max) {
                    (0, Some(1)) => write!(f, "?")?,
                    (0, None) => write!(f, "*")?,
                    (1, None) => write!(f, "+")?,
                    (1, Some(1)) => (),
                    (0, Some(n)) => write!(f, "{{,{}}}", n)?,
                    (n, None) => write!(f, "{{{},}}", n)?,
                    (n, Some(m)) if n == m => write!(f, "{{{}}}", n)?,
                    (l, Some(u)) => write!(f, "{{{},{}}}", l, u)?,
                }
                Ok(())
            }
        }
    }
}
