use core::panic;

use matex_common::{
    error::ParseError,
    node::{BinOp, Expr, Precedence, Statement},
    token::{KeywordType, Token, TokenType},
};

type ParseResult<T> = std::result::Result<T, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    // TODO: Reference from self.tokens instead?
    cur_token: Option<Token>,
    idx: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let first_tok = tokens[0].clone();
        Self {
            tokens,
            cur_token: Some(first_tok),
            idx: 0,
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

        let name = id;
        // TODO: Might not be function definition, could just be function call. Check/peek for TokenType::Equal after
        // matching parenthesis: abs(....(...)) =
        //                          ^---------^ Match this one
        if next_token.typ == TokenType::LeftParenthesis {
            self.expect_identifier("Expected function name")?;
            // Function!
            // parse parameters
            self.expect(
                TokenType::LeftParenthesis,
                "Expected opening parenthesis after function name",
            )?;

            self.parse_parameter_and_type()?;

            let _ = self.expect(
                TokenType::RightParenthesis,
                "Expected closing parenthesis after parameter list!",
            );
            let _ = self.expect(
                TokenType::Equal,
                "Expected assignment operator after function definition...",
            );
        } else {
            return self.parse_statement();
        }

        let function_body = self.parse_expression()?;

        let _ = self.expect(
            TokenType::NewLine,
            "Expected newline after function definition.",
        );

        Ok(Statement::Function {
            name,
            function_body,
        })
    }

    fn parse_parameter_and_type(&mut self) -> ParseResult<(String, Token)> {
        let TokenType::Identifier(param_name) = self.consume()?.typ else {
            panic!("Oh no!");
        };
        // TODO: Do not use expect? => Instead check if type annotation is present to start with.
        self.expect(TokenType::Colon, "Expected colon after parameter name.")?;

        let Ok(type_name) = self.expect_identifier("Expected type name after semicolon.") else {
            panic!("No type name stuff");
        };

        Ok((param_name, type_name))
    }

    fn parse_expression(&mut self) -> ParseResult<Expr> {
        self.parse_precedence(Precedence::None)
    }

    fn parse_precedence(&mut self, prec: Precedence) -> ParseResult<Expr> {
        let token = self.get_token()?;
        let mut node = match token.typ {
            TokenType::Number(_) => self.parse_number()?,
            TokenType::Identifier(id) => self.parse_identifier(id)?,
            TokenType::Keyword(kw) => self.parse_keyword(kw)?,
            TokenType::Minus => todo!(),
            TokenType::LeftParenthesis => self.parse_grouping()?,
            TokenType::LeftSquareBracket => todo!("Implement lists"),
            TokenType::RightSquareBracket => todo!("Implement lists"),
            TokenType::RightBrace => todo!("Implement blocks"),
            TokenType::LeftBrace => todo!("Implement blocks"),
            _ => {
                panic!("\nFailed prefix on: \n{:?}", token);
            }
        };

        println!("\nExpr: {:?}\n", node);
        //println!("Current prec: {:?}", prec);

        println!(
            "Prec: {:?},\nCurrent Token: {},\nToken Precedence: {:?}\n",
            prec,
            self.get_token()?,
            BinOp::from(self.get_token()?.typ).precedence()
        );
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
            //println!("\nBinop: {:?}\n", node);
        }

        //println!("\nReturning node: {:?}\n", node);
        Ok(node)
    }

    fn parse_number(&mut self) -> ParseResult<Expr> {
        let TokenType::Number(n) = self.get_token()?.typ else {
            panic!("shit");
        };

        self.consume()?;
        Ok(Expr::Number(n))
    }

    fn parse_identifier(&mut self, id: String) -> ParseResult<Expr> {
        self.expect_identifier("Expected identifier.")?;
        if self.get_token()?.typ == TokenType::LeftParenthesis {
            // Assume function call
            return self.parse_function_call(id);
        }
        Ok(Expr::Variable(id))
    }

    fn parse_keyword(&mut self, kw: KeywordType) -> ParseResult<Expr> {
        // Expect Keyword type ideally....
        self.consume()?;

        let expr = match kw {
            KeywordType::If => self.parse_if()?,
            KeywordType::Else => todo!(),
        };

        Ok(expr)
    }

    fn parse_if(&mut self) -> ParseResult<Expr> {
        let condition = Box::new(self.parse_expression()?);

        let body = Box::new(self.parse_expression()?);

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
        let right_node = Expr::BinaryOp {
            left: (Expr::Number(-1.0).into()),
            operation: BinOp::Multiply,
            right: (right.into()),
        };

        let node = Expr::BinaryOp {
            left: left.into(),
            operation: BinOp::Add,
            right: right_node.into(),
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
        let right_node = Expr::BinaryOp {
            left: right.into(),
            operation: BinOp::Power,
            right: (Expr::Number(-1.0).into()),
        };

        let node = Expr::BinaryOp {
            left: left.into(),
            operation: BinOp::Multiply,
            right: right_node.into(),
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
        dbg!("parse_assignment");
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
}

impl Parser {
    fn expect(&mut self, tok: TokenType, message: &str) -> ParseResult<Token> {
        let token = self.get_token()?;
        if token.typ != tok {
            // TODO: Change to ParseError?
            panic!("Unexpected token!\n{:?}\n{}", token.typ, message);
        } else {
            self.consume()
        }
    }
    fn expect_identifier(&mut self, message: &str) -> ParseResult<Token> {
        let token = self.get_token()?;
        let TokenType::Identifier(_) = token.typ else {
            return Err(ParseError::WrongToken  {
                message: format!("Expected an identifier: {:?}\n{}", token.typ, message)
            });
        };
        self.consume()?;
        Ok(token)
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

    // TODO: Maybe add consume_error()/similar function
    // to signify a consume that shouldn't result in EndOfStream
    // I.E. in the middle of parsing something that is expected
    fn consume(&mut self) -> ParseResult<Token> {
        let token = self.get_token()?;

        if !self.at_end() {
            self.idx += 1;
            println!("consumed {}", token);

            // TODO: Change to something iterator-like to not have to do this???
            // Also use TokenType::EndOfFile???
            if !self.at_end() {
                self.cur_token = Some(self.tokens[self.idx].clone());
            } else {
                self.cur_token = None
            }
            Ok(token)
        } else {
            self.cur_token = None;
            Err(ParseError::EndOfStream)
        }
    }

    fn consume_newlines(&mut self) -> ParseResult<()> {
        while self.get_token()?.typ == TokenType::NewLine {
            self.consume()?;
        }
        Ok(())
    }

    fn get_token(&self) -> ParseResult<Token> {
        if let Some(token) = self.cur_token.clone() {
            Ok(token)
        } else {
            Err(ParseError::EndOfStream)
        }
    }

    fn peek(&self, offset: i32) -> Option<Token> {
        let index = self.idx as i32 + offset;
        if index < self.tokens.len() as i32 && index >= 0 {
            return Some(self.tokens[index as usize].clone());
        } else {
            return None;
        }
    }

    fn at_end(&self) -> bool {
        self.idx >= self.tokens.len()
    }
}
