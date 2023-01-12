use crate::util::Position;


#[derive(Debug, Clone)]
pub struct Token {
    pub typ: TokenType,
    pub pos: Position,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TokenType {
    Number(f64),
    Identifier(String),

    Plus,
    Minus,
    Star,
    Slash,
    Caret,

    LeftParenthesis,
    RightParenthesis,
    LeftSquareBracket,
    RightSquareBracket,
    RightBrace,
    LeftBrace,

    EndOfFile,
}

impl Token {
    pub fn new(typ: TokenType, pos: Position) -> Token {
        Self { typ, pos }
    }
}

impl PartialEq for TokenType {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}