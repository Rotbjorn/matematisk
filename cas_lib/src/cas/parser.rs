use core::panic;

use super::{token::{Token, TokenType}, node::{Node, OperationType}};

pub struct Parser {
    tokens: Vec<Token>,
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

    pub fn parse(&mut self) -> Node {
        self.parse_expression()
    }
}

impl Parser {
    fn parse_expression(&mut self) -> Node {
        self.parse_precedence(Precedence::None)
    }

    fn parse_precedence(&mut self, prec: Precedence) -> Node {
        let mut node = match self.cur_token.typ {
            TokenType::Number(_) => self.parse_number(),
            TokenType::Identifier(_) => self.parse_identifier(),
            TokenType::Minus => todo!(),
            TokenType::LeftParenthesis => self.parse_grouping(),
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
                TokenType::Plus => self.parse_addition(node),
                TokenType::Minus => self.parse_subtraction(node),
                TokenType::Star => self.parse_multiplication(node),
                TokenType::Slash => self.parse_division(node),
                TokenType::Caret => self.parse_power(node),
                _ => {
                    break;
                }
            };
            //println!("\nBinop: {:?}\n", node);
        }

        //println!("\nReturning node: {:?}\n", node);
        node
    }

    fn parse_number(&mut self) -> Node {
        if let TokenType::Number(n) = self.cur_token.typ {
            self.consume();
            return Node::Number(n);
        } else {
            panic!("Shit");
        }
    }

    fn parse_identifier(&mut self) -> Node {
        if let TokenType::Identifier(s) = self.cur_token.typ.clone() {
            self.consume();
            return Node::Variable(s);
        } else {
            panic!("Oh no.");
        }
    }

    fn parse_grouping(&mut self) -> Node {
        self.consume();
        let node = self.parse_expression();
        self.expect(TokenType::RightParenthesis);

        node
    }

    fn parse_addition(&mut self, left: Node) -> Node {
        self.consume();
        let right = self.parse_precedence(Precedence::Factor);
        let node = Node::BinaryOp {
            left: left.into(),
            operation: OperationType::Add,
            right: right.into(),
        };
        node
    }

    fn parse_subtraction(&mut self, left: Node) -> Node {
        self.consume();
        let right = self.parse_precedence(Precedence::Factor);
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
        node
    }

    fn parse_multiplication(&mut self, left: Node) -> Node {
        self.consume();
        let right = self.parse_precedence(Precedence::Exponent);
        let node = Node::BinaryOp {
            left: left.into(),
            operation: OperationType::Multiply,
            right: right.into(),
        };
        node
    }

    fn parse_division(&mut self, left: Node) -> Node {
        self.consume();
        let right = self.parse_precedence(Precedence::Factor);
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
        node
    }

    fn parse_power(&mut self, left: Node) -> Node {
        // TODO: Something higher than Exponent?
        self.consume();
        let right = self.parse_precedence(Precedence::Exponent);
        let node = Node::BinaryOp {
            left: left.into(),
            operation: OperationType::Power,
            right: right.into(),
        };
        node
    }

    // TODO: Add message parameter for extra information
    fn expect(&mut self, tok: TokenType) -> Result<(), ()> {
        if self.cur_token.typ != tok {
            // TODO: Change to ParseError?
            panic!("Unexpected token!\n{:?}", self.cur_token.typ);
        } else {
            self.consume()
        }
    }

    // TODO: Return if end of tokenstream
    fn consume(&mut self) -> Result<(), ()> {
        self.idx += 1;
        if self.idx < self.tokens.len() {
            println!("Consuming token: {:?}!", self.cur_token);
            self.cur_token = self.tokens[self.idx].clone();
            Ok(())
        } else {
            Err(())
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


// Place function at better place
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
