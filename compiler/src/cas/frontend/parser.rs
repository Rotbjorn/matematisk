use core::panic;
use std::collections::HashMap;

use matex_common::{
    error::ParseError,
    node::{BinOp, Expr, Precedence, Statement},
    token::{KeywordType, Token, TokenType},
};

type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(Default, Debug)]
pub struct ParsedContext {
    functions: HashMap<String, ParsedFunction>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ParsedFunction {
    name: String,
    params: Vec<ParsedParameter>,
    body: Expr,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ParsedParameter {
    name: String,
    type_name: String,
}

pub struct Parser {
    tokens: Vec<Token>,
    // TODO: Reference from self.tokens instead?
    cur_token: Option<Token>,
    idx: usize,

    pub parsed: ParsedContext,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let cur_token = tokens.get(0).cloned();
        Self {
            tokens,
            cur_token,
            idx: 0,
            parsed: ParsedContext::default(),
        }
    }

    pub fn parse(&mut self) -> ParseResult<Statement> {
        let mut nodes: Vec<Statement> = Vec::new();

        loop {
            let result = self.parse_declaration();
            match result {
                Ok(node) => nodes.push(node),
                Err(err) => match &err {
                    ParseError::EndOfStream => break,
                    ParseError::WrongToken { message: _ } => return Err(err),
                },
            }
        }
        Ok(Statement::Program(nodes))
    }
}

impl Parser {
    fn parse_declaration(&mut self) -> ParseResult<Statement> {
        self.consume_newlines()?;

        match self.get_token()?.typ {
            TokenType::Identifier(id) => self.parse_function_possible(id),
            _ => self.parse_statement(),
        }
    }
    fn parse_statement(&mut self) -> ParseResult<Statement> {
        match self.get_token()? {
            _ => Ok(Statement::Expression(self.parse_expression()?)),
        }
    }

    fn parse_function_possible(&mut self, id: String) -> ParseResult<Statement> {
        let Some(next_token) = self.peek(1) else {
            return self.parse_statement();
        };

        let func_name = id;

        // FIXME: Might not be function definition, could just be function call. Check/peek for TokenType::Equal after
        // matching parenthesis: abs(....(...)) =
        //                          ^---------^ Match this one
        if next_token.typ != TokenType::LeftParenthesis {
            return self.parse_statement();
        }

        self.expect_identifier("Expected function name")?;
        // Function!
        // parse parameters
        self.expect(
            TokenType::LeftParenthesis,
            "Expected opening parenthesis after function name",
        )?;

        // TODO: Loop parameter parsing
        let (param_name, type_name) = self.parse_parameter_definition()?;

        self.expect(
            TokenType::RightParenthesis,
            "Expected closing parenthesis after parameter list!",
        )?;

        self.expect(
            TokenType::Equal,
            "Expected assignment operator after function definition...",
        )?;

        let function_body = self.parse_expression()?;

        // TODO: Add function to consume newline OR end of stream.
        // I.E. functions at the end of the file should parse expectedly, and not fail just because no newline character
        let _ = self.expect(
            TokenType::NewLine,
            "Expected newline after function definition.",
        );

        // TODO: A lot of clones...
        self.parsed.functions.insert(
            func_name.clone(),
            ParsedFunction {
                name: func_name.clone(),
                params: vec![ParsedParameter {
                    name: param_name,
                    type_name,
                }],
                body: function_body.clone(),
            },
        );

        Ok(Statement::Function {
            name: func_name,
            body: function_body,
        })
    }

    fn parse_parameter_definition(&mut self) -> ParseResult<(String, String)> {
        let (_, param_name) = self.expect_identifier("Expected parameter name")?;

        // TODO: Do not use expect? => Instead check if type annotation is present to start with.
        self.expect(TokenType::Colon, "Expected colon after parameter name.")?;

        let (_, type_name) = self.expect_identifier("Expected type name after semicolon.")?;

        Ok((param_name, type_name))
    }

    fn parse_expression(&mut self) -> ParseResult<Expr> {
        self.parse_precedence(Precedence::None)
    }

    fn parse_precedence(&mut self, prec: Precedence) -> ParseResult<Expr> {
        let token = self.get_token()?;
        let mut node = match token.typ {
            TokenType::Number(_) => self.parse_number()?,
            TokenType::Identifier(_) => self.parse_identifier()?,
            TokenType::Keyword(kw) => self.parse_keyword(kw)?,
            TokenType::Minus => self.parse_unary_minus()?,
            TokenType::LeftParenthesis => self.parse_grouping()?,
            TokenType::LeftSquareBracket => todo!("Implement lists"),
            TokenType::RightSquareBracket => todo!("Implement lists"),
            TokenType::RightBrace => todo!("Implement blocks"),
            TokenType::LeftBrace => todo!("Implement blocks"),
            _ => {
                panic!("\nFailed prefix on: \n{:?}", token);
            }
        };

        while !self.at_end() && prec <= BinOp::from(self.get_token()?.typ).precedence() {
            node = match self.get_token()?.typ {
                TokenType::Plus => self.parse_addition(node)?,
                TokenType::Minus => self.parse_subtraction(node)?,
                TokenType::Star => self.parse_multiplication(node)?,
                TokenType::Slash => self.parse_division(node)?,
                TokenType::Caret => self.parse_power(node)?,
                TokenType::Less
                | TokenType::LessEqual
                | TokenType::Greater
                | TokenType::GreaterEqual => self.parse_comparison(node)?,
                TokenType::Equal => self.parse_assignment(node)?,
                _ => {
                    break;
                }
            };
        }

        Ok(node)
    }

    fn parse_number(&mut self) -> ParseResult<Expr> {
        let TokenType::Number(n) = self.consume()?.typ else {
            panic!("Expected a number.");
        };

        Ok(Expr::Number(n))
    }

    fn parse_identifier(&mut self) -> ParseResult<Expr> {
        let (_, id) = self.expect_identifier("Expected identifier.")?;

        if self.token_matches(TokenType::LeftParenthesis) {
            // Assume function call
            return self.parse_function_call(id);
        }
        Ok(Expr::Variable(id))
    }

    fn parse_keyword(&mut self, kw: KeywordType) -> ParseResult<Expr> {
        // Expect Keyword type ideally....

        let expr = match kw {
            KeywordType::If => self.parse_if()?,
            KeywordType::Simplify => self.parse_simplify()?,
            _ => {
                panic!("Unhandled: {:?}", kw);
            }
        };

        Ok(expr)
    }
    fn parse_simplify(&mut self) -> ParseResult<Expr> {
        self.expect_keyword(KeywordType::Simplify, "Expected simplify keyword")?;

        let expr = self.parse_expression()?;

        Ok(Expr::Simplify(Box::new(expr)))
    }

    fn parse_if(&mut self) -> ParseResult<Expr> {
        self.expect_keyword(KeywordType::If, "Expected if")?;

        let condition = Box::new(self.parse_expression()?);
        dbg!(&condition);

        self.expect_keyword(KeywordType::Then, "Expected then after if condition.")?;

        let body = Box::new(self.parse_expression()?);
        dbg!(&body);

        self.expect_keyword(KeywordType::Else, "Expected else after if body.")?;

        let else_body = Box::new(self.parse_expression()?);

        Ok(Expr::If {
            condition,
            body,
            else_body,
        })
    }

    fn parse_function_call(&mut self, id: String) -> ParseResult<Expr> {
        self.expect(
            TokenType::LeftParenthesis,
            "Expected parenthesis after function name",
        )?;

        if self.get_token()?.typ == TokenType::RightParenthesis {
            // No arguments passed,
            return Ok(Expr::FunctionCall {
                name: id,
                args: vec![],
            });
        }

        // Create argument vector, and store first argument.
        let mut args: Vec<Expr> = vec![self.parse_expression()?];

        // Parse arguments
        while self.get_token()?.typ == TokenType::Comma {
            self.expect(TokenType::Comma, "Expected comma after argument")?;
            let argument = self.parse_expression()?;
            args.push(argument);
        }
        self.expect(
            TokenType::RightParenthesis,
            "Expected parenthesis after arguments of function call",
        )?;

        Ok(Expr::FunctionCall { name: id, args })
    }

    fn parse_grouping(&mut self) -> ParseResult<Expr> {
        self.expect(TokenType::LeftParenthesis, "Expected opening parenthesis")?;
        let node = self.parse_expression()?;
        self.expect(TokenType::RightParenthesis, "Expected closing parenthesis")?;

        Ok(node)
    }

    fn parse_addition(&mut self, left: Expr) -> ParseResult<Expr> {
        self.expect(TokenType::Plus, "Expected addition operator")?;
        let right = self.parse_precedence(Precedence::Factor)?;
        let node = Expr::BinaryOp {
            left: left.into(),
            operation: BinOp::Add,
            right: right.into(),
        };
        Ok(node)
    }

    fn parse_subtraction(&mut self, left: Expr) -> ParseResult<Expr> {
        self.expect(TokenType::Minus, "Expected subtraction operator")?;
        let right = self.parse_precedence(Precedence::Factor)?;

        let node = Expr::BinaryOp {
            left: Box::new(left),
            operation: BinOp::Subtract,
            right: Box::new(right),
        };
        Ok(node)
    }

    fn parse_multiplication(&mut self, left: Expr) -> ParseResult<Expr> {
        self.expect(TokenType::Star, "Expected multiplication operator")?;
        let right = self.parse_precedence(Precedence::Exponent)?;
        let node = Expr::BinaryOp {
            left: left.into(),
            operation: BinOp::Multiply,
            right: right.into(),
        };
        Ok(node)
    }

    fn parse_division(&mut self, left: Expr) -> ParseResult<Expr> {
        self.consume()?;
        let right = self.parse_precedence(Precedence::Exponent)?;

        let node = Expr::BinaryOp {
            left: Box::new(left),
            operation: BinOp::Divide,
            right: Box::new(right),
        };
        Ok(node)
    }

    fn parse_power(&mut self, left: Expr) -> ParseResult<Expr> {
        // TODO: Something higher than Exponent?
        self.expect(TokenType::Caret, "Expected power operator")?;
        let right = self.parse_precedence(Precedence::Exponent)?;
        let node = Expr::BinaryOp {
            left: Box::new(left),
            operation: BinOp::Power,
            right: Box::new(right),
        };
        Ok(node)
    }

    fn parse_comparison(&mut self, left: Expr) -> ParseResult<Expr> {
        let operation = BinOp::from(self.get_token()?.typ);
        self.consume()?;

        // Support custom infix operators?
        if operation == BinOp::None {
            return Err(ParseError::WrongToken {
                message: "Expected a binary operation...".to_string(),
            });
        }

        let right = self.parse_precedence(Precedence::Term)?;

        Ok(Expr::BinaryOp {
            left: Box::new(left),
            operation,
            right: Box::new(right),
        })
    }

    fn parse_assignment(&mut self, holder: Expr) -> ParseResult<Expr> {
        self.expect(
            TokenType::Equal,
            "Expected assignment operator after variable name.",
        )?;

        let value = self.parse_expression()?;

        Ok(Expr::Assignment {
            holder: Box::new(holder),
            value: Box::new(value),
        })
    }

    fn parse_unary_minus(&mut self) -> ParseResult<Expr> {
        self.expect(TokenType::Minus, "Expected unary minus before expression")?;

        let expr = self.parse_precedence(Precedence::Unary)?;

        Ok(Expr::Unary(Box::new(expr)))
    }
}

impl Parser {
    fn expect(&mut self, expected_type: TokenType, message: &str) -> ParseResult<Token> {
        let actual_typ = self.get_token()?.typ;
        if actual_typ != expected_type {
            // TODO: Change to ParseError
            Err(ParseError::WrongToken {
                message: format!(
                    "{}\nExpected: {:?}\nReceived: {:?}",
                    message, expected_type, actual_typ
                ),
            })
        } else {
            self.consume()
        }
    }

    fn expect_identifier(&mut self, message: &str) -> ParseResult<(Token, String)> {
        let token = self.get_token()?;
        let TokenType::Identifier(ident) = token.typ.clone() else {
            return Err(ParseError::WrongToken  {
                message: format!("Expected an identifier: {:?}\n{}", token.typ, message)
            });
        };
        self.consume()?;
        Ok((token, ident))
    }

    fn expect_keyword(&mut self, expected_kw: KeywordType, message: &str) -> ParseResult<Token> {
        let token = self.get_token()?;
        let TokenType::Keyword(kw) = &token.typ else {
            return Err(ParseError::WrongToken  {
                message: format!("Expected a keyword: {:?}\n{}", token.typ, message)
            });
        };

        if *kw != expected_kw {
            return Err(ParseError::WrongToken {
                message: format!(
                    "Expected a keyword of type {:?}:\n got: {:?}\n{}",
                    expected_kw, kw, message
                ),
            });
        }
        self.consume()?;
        Ok(token)
    }

    fn consume(&mut self) -> ParseResult<Token> {
        let previous_token = self.get_token()?;

        self.idx += 1;
        self.cur_token = self.tokens.get(self.idx).cloned();

        Ok(previous_token)
    }

    fn consume_newlines(&mut self) -> ParseResult<()> {
        while let Ok(token) = self.get_token() {
            if token.typ == TokenType::NewLine {
                self.consume()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn token_matches(&mut self, token_type: TokenType) -> bool {
        let Ok(tok) = self.get_token() else {
            return false;
        };

        // TODO: This is a nice function!
        // self.get_token().is_ok_and(|token| { token.typ == token_type })

        tok.typ == token_type
    }

    fn get_token(&self) -> ParseResult<Token> {
        self.cur_token.clone().ok_or(ParseError::EndOfStream)
    }

    fn peek(&mut self, offset: isize) -> Option<&Token> {
        self.tokens.get((self.idx as isize + offset) as usize)
    }

    fn at_end(&mut self) -> bool {
        self.cur_token.is_none()
    }
}
