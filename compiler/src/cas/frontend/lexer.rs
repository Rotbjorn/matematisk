use core::panic;

use matex_common::token::{KeywordType, Token, TokenType};
use matex_common::util::Position;

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
            if ch.is_numeric() {
                let number: f64 = self.collect_while(|c| c.is_numeric()).parse().unwrap();
                return Token::new(TokenType::Number(number), self.pos);
            }

            if ch.is_alphabetic() {
                let identifier = self.collect_while(|c| c.is_alphabetic());
                // TODO: Extract out from the next_token function, also change to something that allows for I18N?
                let keyword_type = match identifier.as_str() {
                    "if" => Some(KeywordType::If),
                    "else" => Some(KeywordType::Else),
                    _ => None,
                };
                if let Some(keyword) = keyword_type {
                    return Token::new(TokenType::Keyword(keyword), self.pos);
                } else {
                    return Token::new(TokenType::Identifier(identifier), self.pos);
                }
            }

            if let Some(token) = self.special_char() {
                return token;
            }
            panic!(
                "Unhandled character: '{}', Code: {}, Idx: {}",
                ch, ch as usize, self.idx
            )
        } else {
            // TODO: Change end of file to None?
            Token::new(TokenType::EndOfFile, self.pos)
        }
    }

    // TODO: Use something else
    fn collect_while(&mut self, predicate: fn(char) -> bool) -> String {
        let mut buffer = String::new();

        while let Some(ch) = self.peek_char() {
            if predicate(ch) {
                buffer.push(ch);
                self.next_char();
            } else {
                break;
            }
        }
        buffer
    }

    fn skip_whitespace(&mut self) {
        self.collect_while(|c| c == ' ' || c == '\t');
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
        }
        peeked_char
    }

    fn peek_char(&mut self) -> Option<char> {
        if !self.is_eof() {
            // TODO: Change to something with iterators?
            let ch = self.input[self.idx];
            Some(ch)
        } else {
            // End of File
            None
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

                '=' => {
                    let mut token_type = TokenType::Equal;
                    if let Some(next) = self.peek_char() {
                        if next == '=' {
                            token_type = TokenType::EqualEqual
                        }
                    }
                    token_type
                }
                '<' => {
                    let mut token_type = TokenType::Less;
                    if let Some(next) = self.peek_char() {
                        if next == '=' {
                            self.next_char();
                            token_type = TokenType::LessEqual
                        }
                    }
                    token_type
                }
                '>' => {
                    let mut token_type = TokenType::Greater;
                    if let Some(next) = self.peek_char() {
                        if next == '=' {
                            self.next_char();
                            token_type = TokenType::GreaterEqual
                        }
                    }
                    token_type
                }

                ':' => TokenType::Colon,
                ';' => TokenType::Semicolon,

                '.' => TokenType::Dot,
                ',' => TokenType::Comma,

                '(' => TokenType::LeftParenthesis,
                ')' => TokenType::RightParenthesis,
                '[' => TokenType::LeftSquareBracket,
                ']' => TokenType::RightSquareBracket,
                '{' => TokenType::LeftBrace,
                '}' => TokenType::RightBrace,

                '\n' => TokenType::NewLine,
                _ => {
                    // Non handled character, propagate error
                    return None;
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
