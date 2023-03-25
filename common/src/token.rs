use std::fmt::Display;

use crate::util::Position;

#[derive(Debug, Clone)]
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
#[allow(dead_code)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum KeywordType {
    If,
    Else,
}
