use crate::cas::error::ParseError;
use core::panic;

use super::{
    node::{Node, OperationType},
    token::{Token, TokenType},
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

    pub fn parse(&mut self) -> ParseResult<Node> {
        let mut nodes: Vec<Node> = Vec::new();

        loop {
            let result = self.parse_statement();
            match result {
                Ok(node) => nodes.push(node),
                Err(err) => match &err {
                    ParseError::EndOfStream => break,
                    ParseError::WrongToken { message: _ } => return Err(err),
                },
            }
        }
        Ok(Node::Program(nodes))
    }
}

impl Parser {
    fn parse_statement(&mut self) -> ParseResult<Node> {
        match self.get_token()?.typ {
            TokenType::Identifier(_) => return self.parse_function_possible(),
            _ => self.parse_expression(),
        }
    }

    fn parse_function_possible(&mut self) -> ParseResult<Node> {
        let TokenType::Identifier(name) = self.consume()?.typ else {
            return Err(
                ParseError::WrongToken { message: "Expected identifier".to_string() }
            )
        };

        // TODO: Might not be function definition, could just be function call. Check/peek for TokenType::Equal after
        // matching parenthesis: abs(....(...)) =
        //                          ^---------^ Match this one
        if self.get_token()?.typ == TokenType::LeftParenthesis {
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
        }

        let function_body = self.parse_expression()?;

        match self.expect(
            TokenType::NewLine,
            "Expected newline after function definition.",
        ) {
            Ok(_) => {},
            _ => {},
        }

        Ok(Node::Function {
            name,
            function_body: Box::new(function_body),
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

    fn parse_expression(&mut self) -> ParseResult<Node> {
        self.parse_precedence(Precedence::None)
    }

    fn parse_precedence(&mut self, prec: Precedence) -> ParseResult<Node> {
        let token = self.get_token()?;
        let mut node = match token.typ {
            TokenType::Number(_) => self.parse_number()?,
            TokenType::Identifier(_) => self.parse_identifier()?,
            TokenType::Keyword(_) => self.parse_keyword(),
            TokenType::Minus => todo!(),
            TokenType::LeftParenthesis => self.parse_grouping()?,
            TokenType::LeftSquareBracket => todo!(),
            TokenType::RightSquareBracket => todo!(),
            TokenType::RightBrace => todo!(),
            TokenType::LeftBrace => todo!(),
            _ => {
                panic!("\nFailed prefix on: \n{:?}", token);
            }
        };

        //println!("\nNode: {:?}\n", node);
        //println!("Current prec: {:?}", prec);

        while self.idx < self.tokens.len() && prec <= token_precedence(&self.get_token()?.typ) {
            node = match self.get_token()?.typ {
                TokenType::Plus => self.parse_addition(node)?,
                TokenType::Minus => self.parse_subtraction(node)?,
                TokenType::Star => self.parse_multiplication(node)?,
                TokenType::Slash => self.parse_division(node)?,
                TokenType::Caret => self.parse_power(node)?,
                _ => {
                    break;
                }
            };
            //println!("\nBinop: {:?}\n", node);
        }

        //println!("\nReturning node: {:?}\n", node);
        Ok(node)
    }

    fn parse_number(&mut self) -> ParseResult<Node> {
        let TokenType::Number(n) = self.get_token()?.typ else {
            panic!("shit");
        };

        self.consume()?;
        return Ok(Node::Number(n));
    }

    fn parse_identifier(&mut self) -> ParseResult<Node> {
        let TokenType::Identifier(s) = self.consume()?.typ else {
            panic!("Oh no.");
        };
        Ok(Node::Variable(s))
    }

    fn parse_keyword(&self) -> Node {
        todo!()
    }

    fn parse_grouping(&mut self) -> ParseResult<Node> {
        self.expect(TokenType::LeftParenthesis, "Expected opening parenthesis")?;
        let node = self.parse_expression()?;
        self.expect(TokenType::RightParenthesis, "Expected closing parenthesis")?;

        Ok(node)
    }

    fn parse_addition(&mut self, left: Node) -> ParseResult<Node> {
        self.expect(TokenType::Plus, "Expected addition operator")?;
        let right = self.parse_precedence(Precedence::Factor)?;
        let node = Node::BinaryOp {
            left: left.into(),
            operation: OperationType::Add,
            right: right.into(),
        };
        Ok(node)
    }

    fn parse_subtraction(&mut self, left: Node) -> ParseResult<Node> {
        self.expect(TokenType::Minus, "Expected subtraction operator")?;
        let right = self.parse_precedence(Precedence::Factor)?;
        let right_node = Node::BinaryOp {
            left: (Node::Number(-1.0).into()),
            operation: OperationType::Multiply,
            right: (right.into()),
        };

        let node = Node::BinaryOp {
            left: left.into(),
            operation: OperationType::Add,
            right: right_node.into(),
        };
        Ok(node)
    }

    fn parse_multiplication(&mut self, left: Node) -> ParseResult<Node> {
        self.expect(TokenType::Star, "Expected multiplication operator")?;
        let right = self.parse_precedence(Precedence::Exponent)?;
        let node = Node::BinaryOp {
            left: left.into(),
            operation: OperationType::Multiply,
            right: right.into(),
        };
        Ok(node)
    }

    fn parse_division(&mut self, left: Node) -> ParseResult<Node> {
        self.consume()?;
        let right = self.parse_precedence(Precedence::Factor)?;
        let right_node = Node::BinaryOp {
            left: right.into(),
            operation: OperationType::Power,
            right: (Node::Number(-1.0).into()),
        };

        let node = Node::BinaryOp {
            left: left.into(),
            operation: OperationType::Multiply,
            right: right_node.into(),
        };
        Ok(node)
    }

    fn parse_power(&mut self, left: Node) -> ParseResult<Node> {
        // TODO: Something higher than Exponent?
        self.expect(TokenType::Caret, "Expected power operator")?;
        let right = self.parse_precedence(Precedence::Exponent)?;
        let node = Node::BinaryOp {
            left: left.into(),
            operation: OperationType::Power,
            right: right.into(),
        };
        Ok(node)
    }

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

    // TODO: Maybe add consume_error()/similar function
    // to signify a consume that shouldn't result in EndOfStream
    // I.E. in the middle of parsing something that is expected
    fn consume(&mut self) -> ParseResult<Token> {
        if self.idx < self.tokens.len() {
            let opt_token = self.cur_token.clone();
            self.idx += 1;
            println!("Consuming token: {:?}!", self.cur_token);

            // TODO: Change to something iterator-like to not have to do this???
            // Also use TokenType::EndOfFile???
            if self.idx < self.tokens.len() {
                self.cur_token = Some(self.tokens[self.idx].clone());
            } else {
                self.cur_token = None
            }
            let Some(token) = opt_token else {
                panic!("NOOO");
            };
            Ok(token)
        } else {
            self.cur_token = None;
            Err(ParseError::EndOfStream)
        }
    }

    fn get_token(&self) -> ParseResult<Token> {
        if let Some(token) = self.cur_token.clone() {
            Ok(token)
        } else {
            Err(ParseError::EndOfStream)
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
enum Precedence {
    None,
    Term,
    Factor,
    Exponent,
}

// TODO: Place function at better place
fn token_precedence(typ: &TokenType) -> Precedence {
    use Precedence::*;
    use TokenType::*;
    return match typ {
        Plus | Minus => Term,
        Star | Slash => Factor,
        Caret => Exponent,
        _ => None,
    };
}
