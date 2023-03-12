use core::panic;
use crate::cas::error::ParseError;

use super::{token::{Token, TokenType}, node::{Node, OperationType}};

type ParseResult<T> = std::result::Result<T, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    // TODO: Reference from self.tokens instead?
    cur_token: Token,
    idx: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let first_tok = tokens[0].clone();
        Self {
            tokens,
            cur_token: first_tok,
            idx: 0,
        }
    }

    pub fn parse(&mut self) -> ParseResult<Node> {
        let mut nodes: Vec<Node> = Vec::new();

        loop {
            let result = self.parse_statement();
            match result {
                Ok(node) => nodes.push(node),
                Err(err) => {
                    match &err {
                        ParseError::EndOfStream => {
                            break
                        },
                        ParseError::WrongToken { message: _ } => return Err(err)

                    }
                }
            }
        }
        return Ok(Node::Program(nodes))
    }


}

impl Parser {
    fn parse_statement(&mut self) -> ParseResult<Node> {
        match self.cur_token.typ {
            TokenType::Identifier(_) => return self.parse_function_possible(),
            _ => return self.parse_expression(),
        }
    }

    fn parse_function_possible(&mut self) -> ParseResult<Node> {
        let TokenType::Identifier(name) = self.cur_token.typ.clone() else {
            // FIX?
            self.expect(TokenType::Caret, "message")?;
            panic!("Smack");
        };
        self.consume()?;

        if self.cur_token.typ == TokenType::LeftParenthesis {
            // Function!
            // parse parameters
            self.consume()?;

            self.parse_parameter_and_type()?;

            let _ = self.expect(TokenType::RightParenthesis, "Expected closing parenthesis after parameter list!");
            let _ = self.expect(TokenType::Equal, "Expected assignment operator after function definition...");
        }

        let function_body = self.parse_expression()?; 

        Ok(Node::Function {
            name, 
            function_body: Box::new(function_body)
        })
    }

    fn parse_parameter_and_type(&mut self) -> ParseResult<(String, Token)> {
        let TokenType::Identifier(param_name) = self.cur_token.typ.clone() else {
            panic!("Oh no!");
        };
        self.consume()?;
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
        let mut node = match self.cur_token.typ {
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
                panic!("\nFailed prefix on: \n{:?}", self.cur_token);
            }
        };

        //println!("\nNode: {:?}\n", node);
        //println!("Current prec: {:?}", prec);

        while self.idx < self.tokens.len() && prec <= token_precedence(&self.cur_token.typ) {
            node = match self.cur_token.typ {
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
        let TokenType::Number(n) = self.cur_token.typ else {
            panic!("shit");
        };

        self.consume()?;
        return Ok(Node::Number(n))
    }

    fn parse_identifier(&mut self) -> ParseResult<Node> {
        let TokenType::Identifier(s) = self.cur_token.typ.clone() else {
            panic!("Oh no.");
        };
        self.consume()?;
        return Ok(Node::Variable(s))
    }

    fn parse_keyword(&self) -> Node {
        todo!()
    }

    fn parse_grouping(&mut self) -> ParseResult<Node> {
        self.consume()?;
        let node = self.parse_expression()?;
        self.expect(TokenType::RightParenthesis, "Expected closing parenthesis")?;

        Ok(node)
    }

    fn parse_addition(&mut self, left: Node) -> ParseResult<Node> {
        self.consume()?;
        let right = self.parse_precedence(Precedence::Factor)?;
        let node = Node::BinaryOp {
            left: left.into(),
            operation: OperationType::Add,
            right: right.into(),
        };
        Ok(node)
    }

    fn parse_subtraction(&mut self, left: Node) -> ParseResult<Node> {
        self.consume()?;
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
        self.consume()?;
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
        self.consume()?;
        let right = self.parse_precedence(Precedence::Exponent)?;
        let node = Node::BinaryOp {
            left: left.into(),
            operation: OperationType::Power,
            right: right.into(),
        };
        Ok(node)
    }

    fn expect(&mut self, tok: TokenType, message: &str) -> ParseResult<Token> {
        if self.cur_token.typ != tok {
            // TODO: Change to ParseError?
            panic!("Unexpected token!\n{:?}\n{}", self.cur_token.typ, message);
        } else {
            self.consume()
        }
    }
    fn expect_identifier(&mut self, message: &str) -> ParseResult<Token> {
        let TokenType::Identifier(_) = self.cur_token.typ.clone() else {
            return Err(ParseError::WrongToken  {
                message: format!("Expected an identifier: {:?}\n{}", self.cur_token.typ, message)
            });
        };
        let token = self.cur_token.clone();
        self.consume()?;
        Ok(token)
    }

    // TODO: Return if end of tokenstream
    fn consume(&mut self) -> ParseResult<Token> {
        let token = self.cur_token.clone();
        if self.idx < self.tokens.len() {
            self.idx += 1;
            println!("Consuming token: {:?}!", self.cur_token);
            // TODO: Change to something iterator-like to not have to do this???
            if self.idx < self.tokens.len() {
                self.cur_token = self.tokens[self.idx].clone();
            }
            Ok(token)
        } else {
            println!("We last consumed: {:?}", token);
            return Err(ParseError::EndOfStream);
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
    use TokenType::*;
    use Precedence::*;
    return match typ {
        Plus | Minus => Term,
        Star | Slash => Factor,
        Caret => Exponent,
        _ => None,
    };
}
