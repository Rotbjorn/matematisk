use std::fmt::Display;

use crate::token::{KeywordType, Token, TokenType};

// TODO: Contain positon where error occured?
#[derive(Debug, Clone)]
pub enum ParseError {
    WrongToken {
        message: String,
        expected: TokenType,
        actual: Token,
    },
    WrongKeyword {
        message: String,
        expected: KeywordType,
        actual: Token,
    },
    NotIdentifier {
        message: String,
        actual: Token,
    },
    NotComparison {
        message: String,
        actual: Token,
    },
    EndOfStream,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParseError::*;
        match self {
            WrongToken {
                message,
                expected: _,
                actual: _,
            }
            | WrongKeyword {
                message,
                expected: _,
                actual: _,
            } => f.write_str(message),
            NotIdentifier { message, actual: _ } => f.write_str(message),
            NotComparison { message, actual: _ } => f.write_str(message),
            EndOfStream => f.write_str("End of stream"),
        }
    }
}
