use core::panic;

use log::{debug, error};
use matex_common::{
    error::ParseError,
    function::{Function, Parameter},
    node::{BinOp, Expr, Precedence, Program, Statement},
    token::{KeywordType, Token, TokenType},
    util::SymbolTable,
};

macro_rules! parser_debug {
    ($($arg:tt)+) => (debug!(target: "matex::parser", "[{}:{}] {}", file!(), line!(), &format!($($arg)+)));
}

macro_rules! parser_error {
    ($($arg:tt)+) => (error!(target: "matex::parser", "[{}:{}] {}", file!(), line!(), &format!($($arg)+)));
}

type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(Default, Debug)]
pub struct Context {
    pub functions: SymbolTable<Function>,
}

#[derive(Debug)]
#[allow(dead_code)]

pub struct Parser {
    tokens: Vec<Token>,
    // TODO: Reference from self.tokens instead?
    cur_token: Option<Token>,
    idx: usize,

    pub parsed: Context,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let cur_token = tokens.get(0).cloned();
        Self {
            tokens,
            cur_token,
            idx: 0,
            parsed: Context::default(),
        }
    }

    pub fn parse(&mut self) -> ParseResult<Program> {
        use ParseError::*;
        let mut nodes: Vec<Statement> = Vec::new();

        loop {
            let result = self.parse_declaration();
            match result {
                Ok(node) => nodes.push(node),
                Err(err) => match &err {
                    EndOfStream => break,
                    _ => {
                        parser_error!("Failed parsing");
                        parser_error!("----------ERROR----------");
                        parser_error!("{}", err);
                        parser_error!("-------------------------");
                        return Err(err);
                    }
                },
            }
        }
        Ok(Program(nodes))
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
        parser_debug!("Parsing statement");
        match self.get_token()?.typ {
            TokenType::Tilde => self.parse_unset_statement(),
            _ => {
                let expression = Statement::Expression(self.parse_expression()?);
                self.consume_newline_or_eof("Expected newline after expression statement.")?;
                Ok(expression)
            }
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

        let param = self.parse_parameter_definition()?;
        let mut params = vec![param];

        while let TokenType::Comma = self.get_token()?.typ {
            self.consume()?;
            let param = self.parse_parameter_definition()?;
            params.push(param);
        }

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
        self.consume_newline_or_eof("Expected newline after function definition")?;
        /* PREVIOUS CODE:
        let _ = self.expect(
            TokenType::NewLine,
            "Expected newline after function definition.",
        );*/

        // TODO: A lot of clones...
        self.parsed.functions.insert(
            func_name.clone(),
            Function {
                name: func_name.clone(),
                params: params.clone(),
                body: function_body.clone(),
            },
        );

        Ok(Statement::FunctionDefinition(Function {
            name: func_name,
            params: params.clone(),
            body: function_body,
        }))
    }

    fn parse_parameter_definition(&mut self) -> ParseResult<Parameter> {
        parser_debug!("Parsing parameter definition");
        let (_, param_name) = self.expect_identifier("Expected parameter name")?;

        let mut param = Parameter {
            name: param_name,
            type_name: "".to_string(),
        };

        let Some(next) = self.peek(0) else {
            return Ok(param);
        };

        if next.typ != TokenType::Colon {
            return Ok(param);
        }

        self.expect(TokenType::Colon, "Expected colon after parameter name.")?;
        let (_, type_name) = self.expect_identifier("Expected type name after semicolon.")?;

        param.type_name = type_name;
        Ok(param)
    }

    fn parse_unset_statement(&mut self) -> ParseResult<Statement> {
        parser_debug!("Parsing unset statement");
        self.expect(TokenType::Tilde, "Expected unset operator")?;
        let (_, symbol) = self.expect_identifier("Expected symbol after unset operator")?;

        let statement = Statement::UnsetVariable(symbol);

        parser_debug!("Returning {:?}", statement);

        Ok(statement)
    }

    fn parse_expression(&mut self) -> ParseResult<Expr> {
        parser_debug!("Parsing expression");
        self.parse_precedence(Precedence::None)
    }

    fn parse_precedence(&mut self, prec: Precedence) -> ParseResult<Expr> {
        use TokenType::*;

        parser_debug!("Parsing expression with precedence: {:?}", prec);

        let token = self.get_token()?;

        let mut node = match token.typ {
            Number(_) => self.parse_number()?,
            Identifier(_) => self.parse_identifier()?,
            Keyword(kw) => self.parse_keyword(kw)?,
            Minus => self.parse_unary_minus()?,
            LeftParenthesis => self.parse_grouping()?,
            LeftSquareBracket => self.parse_vector()?,
            RightBrace => todo!("Implement blocks"),
            LeftBrace => todo!("Implement blocks"),
            Plus | Star | Slash | Caret | Equal | EqualEqual | Less | Greater | LessEqual
            | GreaterEqual | Colon | Semicolon | Tilde | Dot | Comma | RightParenthesis
            | NewLine | RightSquareBracket => panic!("Failed prefix on: {}", token),
        };

        while !self.at_end() && prec <= BinOp::from(&self.get_token()?.typ).precedence() {
            node = match self.get_token()?.typ {
                Plus => self.parse_addition(node)?,
                Minus => self.parse_subtraction(node)?,
                Star => self.parse_multiplication(node)?,
                Slash => self.parse_division(node)?,
                Caret => self.parse_power(node)?,
                EqualEqual | Less | LessEqual | Greater | GreaterEqual => {
                    self.parse_comparison(node)?
                }
                Equal => self.parse_assignment(node)?,
                _ => {
                    // TODO: Support custom infix operators?
                    break;
                }
            };
        }

        Ok(node)
    }

    fn parse_number(&mut self) -> ParseResult<Expr> {
        parser_debug!("Parsing number");
        let TokenType::Number(n) = self.consume()?.typ else {
            panic!("Expected a number.");
        };

        let number = Expr::Number(n);

        parser_debug!("Returning {:?}", number);

        Ok(number)
    }

    fn parse_identifier(&mut self) -> ParseResult<Expr> {
        parser_debug!("Parsing identifier");
        let (_, id) = self.expect_identifier("Expected identifier.")?;

        if self.token_matches(TokenType::LeftParenthesis) {
            // Assume function call
            return self.parse_function_call(id);
        }

        let variable = Expr::Variable(id);

        parser_debug!("Variable {:?}", variable);

        Ok(variable)
    }

    fn parse_keyword(&mut self, kw: KeywordType) -> ParseResult<Expr> {
        parser_debug!("Parsing keyword {:?}", kw);

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
        parser_debug!("Parsing simplify");
        self.expect_keyword(KeywordType::Simplify, "Expected simplify keyword")?;

        let expr = self.parse_expression()?;

        let simplify = Expr::Simplify(Box::new(expr));
        parser_debug!("Simplify {:?}", simplify);
        Ok(simplify)
    }

    fn parse_if(&mut self) -> ParseResult<Expr> {
        parser_debug!("Parsing if expression");
        self.expect_keyword(KeywordType::If, "Expected if")?;

        let condition = Box::new(self.expect_expression(Precedence::None)?);

        self.expect_keyword(KeywordType::Then, "Expected then after if condition.")?;

        let body = Box::new(self.expect_expression(Precedence::None)?);

        self.expect_keyword(KeywordType::Else, "Expected else after if body.")?;

        let else_body = Box::new(self.expect_expression(Precedence::None)?);

        let if_expr = Expr::If {
            condition,
            body,
            else_body,
        };

        parser_debug!("Returning {:?}", if_expr);

        Ok(if_expr)
    }

    fn parse_function_call(&mut self, id: String) -> ParseResult<Expr> {
        parser_debug!("Parsing function call");
        self.expect(
            TokenType::LeftParenthesis,
            "Expected parenthesis after function name",
        )?;

        if self.get_token()?.typ == TokenType::RightParenthesis {
            parser_debug!("No arguments passed");
            return Ok(Expr::FunctionCall {
                name: id,
                args: vec![],
            });
        }

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

        let function_call = Expr::FunctionCall { name: id, args };
        parser_debug!("Returning {:?}", function_call);
        Ok(function_call)
    }

    fn parse_vector(&mut self) -> ParseResult<Expr> {
        self.expect(
            TokenType::LeftSquareBracket,
            "Expected opening square bracket for vector",
        )?;

        let expression = self.parse_expression()?;
        let mut expressions: Vec<Expr> = Vec::from([expression]);

        while self.get_token()?.typ == TokenType::Comma {
            self.consume()?;
            expressions.push(self.parse_expression()?);
        }

        self.expect(
            TokenType::RightSquareBracket,
            "Expected closing square bracket for vector",
        )?;

        Ok(Expr::Vector(expressions))
    }

    fn parse_grouping(&mut self) -> ParseResult<Expr> {
        self.expect(TokenType::LeftParenthesis, "Expected opening parenthesis")?;
        let node = self.expect_expression(Precedence::None)?;
        self.expect(TokenType::RightParenthesis, "Expected closing parenthesis")?;

        Ok(node)
    }

    fn parse_addition(&mut self, left: Expr) -> ParseResult<Expr> {
        parser_debug!("Parsing addition");
        self.expect(TokenType::Plus, "Expected addition operator")?;
        let right = self.expect_expression(Precedence::Factor)?;
        let node = Expr::BinaryOp {
            left: left.into(),
            operation: BinOp::Add,
            right: right.into(),
        };

        parser_debug!("Returning addition {:?}", node);
        Ok(node)
    }

    fn parse_subtraction(&mut self, left: Expr) -> ParseResult<Expr> {
        parser_debug!("Parsing subtraction");
        self.expect(TokenType::Minus, "Expected subtraction operator")?;
        let right = self.expect_expression(Precedence::Factor)?;

        let node = Expr::BinaryOp {
            left: Box::new(left),
            operation: BinOp::Subtract,
            right: Box::new(right),
        };

        parser_debug!("Returning subtraction {:?}", node);
        Ok(node)
    }

    fn parse_multiplication(&mut self, left: Expr) -> ParseResult<Expr> {
        parser_debug!("Parsing multiplication");
        self.expect(TokenType::Star, "Expected multiplication operator")?;
        let right = self.expect_expression(Precedence::Exponent)?;
        // let Ok(right) = right else {
        // parser_error!("ERROR: {:?}", right.clone().err().unwrap());
        // return right;
        // };
        let node = Expr::BinaryOp {
            left: left.into(),
            operation: BinOp::Multiply,
            right: right.into(),
        };

        parser_debug!("Returning multiplication {:?}", node);
        Ok(node)
    }


    // TODO: Merge with parse_multiplication
    fn parse_division(&mut self, left: Expr) -> ParseResult<Expr> {
        parser_debug!("Parsing division");
        self.consume()?;
        let right = self.expect_expression(Precedence::Exponent)?;

        let node = Expr::BinaryOp {
            left: Box::new(left),
            operation: BinOp::Divide,
            right: Box::new(right),
        };

        parser_debug!("Returning division {:?}", node);
        Ok(node)
    }

    fn parse_power(&mut self, left: Expr) -> ParseResult<Expr> {
        parser_debug!("Parsing power");
        // TODO: Something higher than Exponent?
        self.expect(TokenType::Caret, "Expected power operator")?;
        let right = self.expect_expression(Precedence::Exponent)?;

        let node = Expr::BinaryOp {
            left: Box::new(left),
            operation: BinOp::Power,
            right: Box::new(right),
        };

        parser_debug!("Returning power {:?}", node);
        Ok(node)
    }

    fn parse_comparison(&mut self, left: Expr) -> ParseResult<Expr> {
        parser_debug!("Parsing comparison");
        let token = self.get_token()?;
        let operation = BinOp::from(&token.typ);

        self.consume()?;

        // Support custom infix operators?
        if operation == BinOp::None {
            return Err(ParseError::NotComparison {
                message: "Expected a comparison operator".to_owned(),
                actual: token,
            });
        }

        let right = self.expect_expression(Precedence::Term)?;

        Ok(Expr::BinaryOp {
            left: Box::new(left),
            operation,
            right: Box::new(right),
        })
    }

    fn parse_assignment(&mut self, holder: Expr) -> ParseResult<Expr> {
        parser_debug!("Parsing assignment");
        self.expect(
            TokenType::Equal,
            "Expected assignment operator after variable name.",
        )?;

        let value = self.expect_expression(Precedence::None)?;

        Ok(Expr::Assignment {
            holder: Box::new(holder),
            value: Box::new(value),
        })
    }

    fn parse_unary_minus(&mut self) -> ParseResult<Expr> {
        parser_debug!("Parsing unary minus");
        self.expect(TokenType::Minus, "Expected unary minus before expression")?;

        let expr = self.expect_expression(Precedence::Unary)?;

        let unary = Expr::Unary(Box::new(expr));

        debug!("Returning {:?}", unary);

        Ok(unary)
    }
}

impl Parser {
    fn expect_expression(&mut self, prec: Precedence) -> ParseResult<Expr> {
        let expr = self.parse_precedence(prec);
        if let Err(ParseError::EndOfStream) = expr {
            return Err(ParseError::UnexpectedEndOfStream {
                message: "Expected an expression".to_string(),
            });
        }
        expr
    }

    fn expect(&mut self, expected_type: TokenType, message: &str) -> ParseResult<Token> {
        parser_debug!("Expecting {:?}", expected_type);

        let token = self.get_token()?;

        if token.typ != expected_type {
            parser_debug!("Failed expect");
            Err(ParseError::WrongToken {
                message: message.to_owned(),
                expected: expected_type,
                actual: token,
            })
        } else {
            self.consume()
        }
    }

    fn expect_identifier(&mut self, message: &str) -> ParseResult<(Token, String)> {
        parser_debug!("Expecting identifier");
        let token = self.get_token()?;
        let TokenType::Identifier(ident) = token.typ.clone() else {
            parser_debug!("Failed expect identifier, got {:?} instead", token.typ);
            return Err(ParseError::NotIdentifier {
                message: message.to_owned(),
                actual: token,
            });
        };
        self.consume()?;
        Ok((token, ident))
    }

    fn expect_keyword(&mut self, expected_kw: KeywordType, message: &str) -> ParseResult<Token> {
        parser_debug!("Expecting keyword {:?}", expected_kw);
        let token = self.get_token()?;
        let TokenType::Keyword(kw) = &token.typ else {
            parser_debug!("Failed expect keyword, got {:?} instead", token.typ);
            return Err(ParseError::WrongKeyword { message: message.to_owned(), expected: expected_kw, actual: token });
        };

        if *kw != expected_kw {
            return Err(ParseError::WrongKeyword {
                message: message.to_owned(),
                expected: *kw,
                actual: token,
            });
        }
        self.consume()?;
        Ok(token)
    }

    fn consume(&mut self) -> ParseResult<Token> {
        let previous_token = self.get_token()?;

        parser_debug!("Consumed {}", previous_token);

        self.idx += 1;
        self.cur_token = self.tokens.get(self.idx).cloned();

        Ok(previous_token)
    }

    fn consume_newlines(&mut self) -> ParseResult<()> {
        parser_debug!("Consuming newlines");
        while let Ok(token) = self.get_token() {
            if token.typ == TokenType::NewLine {
                self.consume()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn consume_newline_or_eof(&mut self, message: &str) -> ParseResult<()> {
        if let Err(e) = self.expect(TokenType::NewLine, message) {
            return if let ParseError::EndOfStream = e {
                Ok(())
            } else {
                Err(e)
            };
        }
        Ok(())
    }

    fn token_matches(&mut self, token_type: TokenType) -> bool {
        self.get_token().is_ok_and(|token| { token.typ == token_type })
    }

    fn get_token(&self) -> ParseResult<Token> {
        self.cur_token.clone().ok_or(ParseError::EndOfStream)
    }

    fn peek(&mut self, offset: isize) -> Option<&Token> {
        self.tokens.get((self.idx as isize + offset) as usize)
    }

    fn at_end(&mut self) -> bool {
        parser_debug!("Check if at end");
        self.cur_token.is_none()
    }
}
