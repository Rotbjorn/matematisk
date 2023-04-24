use std::{fmt::Display, str::FromStr};

use crate::util::Position;
#[cfg(target_arch = "wasm32")]
use serde::Serialize;

#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", derive(Serialize))]
pub struct Token {
    pub typ: TokenType,
    pub pos: Position,
}

impl Token {
    pub fn new(typ: TokenType, pos: Position) -> Token {
        Self { typ, pos }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ {:?}, {} }}", self.typ, self.pos))
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", derive(Serialize))]
pub enum TokenType {
    Number(f64),
    Identifier(String),
    Keyword(KeywordType),

    Plus,
    Minus,
    Star,
    Slash,
    Caret,

    Equal,
    EqualEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    Colon,
    Semicolon,

    Tilde,

    Dot,
    Comma,

    LeftParenthesis,
    RightParenthesis,
    LeftSquareBracket,
    RightSquareBracket,
    RightBrace,
    LeftBrace,

    NewLine,
}

impl PartialEq for TokenType {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Serialize))]
pub enum KeywordType {
    If,
    Else,
    Then,
    Simplify,
}

impl FromStr for KeywordType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "if" => Ok(KeywordType::If),
            "else" => Ok(KeywordType::Else),
            "then" => Ok(KeywordType::Then),
            "simplify" => Ok(KeywordType::Simplify),
            _ => Err(()),
        }
    }
}
