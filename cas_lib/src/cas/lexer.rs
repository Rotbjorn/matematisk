use core::panic;
use crate::util::Position;

use super::token::{Token, TokenType};

#[derive(Debug)]
pub struct Lexer {
    pub input: Vec<char>,
    pub idx: usize,
    pub pos: Position,
}


impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            idx: 0,
            pos: Position { row: 1, col: 0 },
        }
    }
}

impl Lexer {
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if let Some(ch) = self.peek_char() {

            if ch.is_numeric(){
                let number: f64 = self.collect_while(|c| c.is_numeric()).parse().unwrap();
                return Token::new(TokenType::Number(number), self.pos);
            }

            if ch.is_alphabetic() {
                let identifier = self.collect_while(|c| c.is_alphabetic());
                return Token::new(TokenType::Identifier(identifier), self.pos);
            }



            if let Some(token) = self.special_char() {
                return token
            } 
            panic!("Unhandled character: '{}', Code: {}, Idx: {}", ch, ch as usize, self.idx)

        } else {
            // TODO: Change end of file to None?
            return Token::new(TokenType::EndOfFile, self.pos)
        }
    }

    fn collect_while(&mut self, predicate: fn(char) -> bool) -> String
    {
        let mut buffer = String::new();

        loop {
            if let Some(ch) = self.peek_char() {
                if predicate(ch) {
                    buffer.push(ch);
                    // TODO: Use next_char()?
                    self.next_char();
                } else {
                    // No longer fits the predicate
                    break;
                }
            } else {
                // End of File
                break;
            }
        }
        buffer
    }

    fn skip_whitespace(&mut self) {
        self.collect_while(|c| c == ' ' || c == '\n');
    }

    fn next_char(&mut self) -> Option<char> {
        let peeked_char = self.peek_char();
        if let Some(ch) = peeked_char {
            self.idx += 1;

            if ch != '\n' {
                self.pos.col += 1;
            } else {
                self.pos.row += 1;
                self.pos.col = 0;
            }

            return peeked_char;
        } else {
            None
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        if !self.is_eof() {
            let ch = self.input[self.idx];
            return Some(ch);
        } else {
            // End of File
            return None;
        }
    }

    fn is_eof(&self) -> bool {
        self.idx >= self.input.len()
    }

    fn special_char(&mut self) -> Option<Token> {
        let token_type: TokenType;
        if let Some(ch) = self.next_char() {

            token_type = match ch {
                '+' => TokenType::Plus, 
                '-' => TokenType::Minus,
                '*' => TokenType::Star,
                '/' => TokenType::Slash,
                '^' => TokenType::Caret,

                '(' => TokenType::LeftParenthesis,
                ')' => TokenType::RightParenthesis,
                '[' => TokenType::LeftSquareBracket,
                ']' => TokenType::RightSquareBracket,
                '{' => TokenType::LeftBrace,
                '}' => TokenType::RightBrace,

                _ => {
                    // Non handled character, raise up error
                    return None
                }
            }
        } else {
            token_type = TokenType::EndOfFile;
        }
        Some(Token::new(token_type, self.pos))
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if let TokenType::EndOfFile = token.typ {
            None
        } else {
            Some(token)
        }
    }
}