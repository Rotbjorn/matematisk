use std::fmt::{Error, Write};

use crate::token::TokenType;

type Statements = Vec<Statement>;

#[derive(Debug, Clone)]
pub enum Statement {
    Program(Statements),
    Function { name: String, function_body: Expr },
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    Variable(String),

    List(Vec<Expr>),

    // TODO: Break into addition, multiply, power
    BinaryOp {
        left: Box<Expr>,
        operation: BinOp,
        right: Box<Expr>,
    },

    Assignment {
        holder: Box<Expr>,
        value: Box<Expr>,
    },

    If {
        condition: Box<Expr>,
        body: Box<Expr>,
        else_body: Box<Expr>,
    },

    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,

    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    Assignment,

    None,
}

impl BinOp {
    pub fn precedence(self) -> Precedence {
        match self {
            BinOp::Add | BinOp::Subtract => Precedence::Term,
            BinOp::Multiply | BinOp::Divide => Precedence::Factor,
            BinOp::Power => Precedence::Exponent,

            BinOp::Less | BinOp::LessEqual | BinOp::Greater | BinOp::GreaterEqual => {
                Precedence::Comparison
            }

            BinOp::Assignment => Precedence::Assignment,

            BinOp::None => Precedence::None,
        }
    }
}

impl From<TokenType> for BinOp {
    fn from(value: TokenType) -> Self {
        match value {
            TokenType::Plus => BinOp::Add,
            TokenType::Minus => BinOp::Subtract,
            TokenType::Star => BinOp::Multiply,
            TokenType::Slash => BinOp::Divide,
            TokenType::Caret => BinOp::Power,
            TokenType::Less => BinOp::Less,
            TokenType::LessEqual => BinOp::LessEqual,
            TokenType::Greater => BinOp::Greater,
            TokenType::GreaterEqual => BinOp::GreaterEqual,
            _ => BinOp::None,
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
pub enum Precedence {
    None,
    Assignment,
    Comparison,
    Term,
    Factor,
    Exponent,
}

pub trait Visitor<T> {
    fn visit_statement(&mut self, n: &Statement) -> T;
    fn visit_expr(&mut self, n: &Expr) -> T;
}

pub struct ASTGraphGenerator<'a, W: Write> {
    count: u32,
    out: &'a mut W,
}

impl<'a, W: Write> ASTGraphGenerator<'a, W> {
    pub fn new(out: &'a mut W) -> Self {
        Self { count: 0, out }
    }

    pub fn create_dot_graph(&mut self, stmt: &Statement) -> Result<(), Error> {
        self.out.write_str("digraph AST {\n")?;
        self.out.write_str("\tlabel = \"Abstract Syntax Tree\"\n")?;
        self.out.write_str("\tfontname = \"Arial\"\n")?;
        self.out.write_str("\tnode [fontname = \"Arial\"]\n")?;
        self.out.write_str("\tedge [fontname = \"Arial\"]\n\n")?;

        self.visit_statement(stmt)?;

        self.out.write_char('}')?;

        Ok(())
    }
}

// TODO: There is a library for creating Graphviz dot files!
impl<'a, W: Write> Visitor<Result<u32, Error>> for ASTGraphGenerator<'a, W> {
    fn visit_statement(&mut self, statement: &Statement) -> Result<u32, Error> {
        let current_node_id = self.count;
        match statement {
            Statement::Program(v) => {
                writeln!(self.out, "\tN_{} [label = \"<>\"]", current_node_id)?;
                self.count += 1;
                for (index, node) in v.iter().enumerate() {
                    let target_id = self.visit_statement(node)?;
                    writeln!(
                        self.out,
                        "\tN_{} -> N_{} [label = \"{}\"]",
                        current_node_id, target_id, index
                    )?;
                }
            }

            Statement::Function {
                name: function_name,
                function_body,
            } => {
                writeln!(
                    self.out,
                    "\tN_{} [label = \"func: {}\"]",
                    current_node_id, function_name
                )?;
                self.count += 1;

                let body_id = self.visit_expr(function_body)?;

                writeln!(self.out, "\tN_{} ->  N_{}", current_node_id, body_id)?;
            }
            Statement::Expression(expr) => {
                let _target_id = self.visit_expr(expr);
                self.count += 1;
            }
        }

        Ok(current_node_id)
    }

    fn visit_expr(&mut self, expr: &Expr) -> Result<u32, Error> {
        let current_node_id = self.count;
        match expr {
            Expr::List(v) => {
                writeln!(self.out, "\tN_{} [label = \"<>\"]", current_node_id)?;
                self.count += 1;
                for (index, node) in v.iter().enumerate() {
                    let target_id = self.visit_expr(node)?;
                    writeln!(
                        self.out,
                        "\tN_{} -> N_{} [label = \"{}\"]\n",
                        current_node_id, target_id, index
                    )?;
                }
            }
            Expr::Number(n) => {
                writeln!(self.out, "\tN_{} [label = \"num: {}\"]", current_node_id, n)?;
                self.count += 1;
            }
            Expr::Variable(s) => {
                writeln!(self.out, "\tN_{} [label = \"var: {}\"]", current_node_id, s)?;
                self.count += 1;
            }
            Expr::BinaryOp {
                left,
                operation,
                right,
            } => {
                writeln!(
                    self.out,
                    "\tN_{} [label = \"{:?}\"]",
                    current_node_id, operation
                )?;
                self.count += 1;

                let lhs_id = self.visit_expr(left)?;
                let rhs_id = self.visit_expr(right)?;

                if *operation == BinOp::Power {
                    writeln!(
                        self.out,
                        "\tN_{0} -> N_{1} [label = \"base\"]\nN_{0} -> N_{2} [label = \"exp\"]",
                        current_node_id, lhs_id, rhs_id
                    )?;
                } else {
                    writeln!(
                        self.out,
                        "\tN_{0} -> N_{1} [label = \"lhs\"]\n\tN_{0} -> N_{2} [label = \"rhs\"]",
                        current_node_id, lhs_id, rhs_id
                    )?;
                }
            }

            Expr::Assignment { holder, value } => {
                writeln!(self.out, "\tN_{} [label = \"Assignment\"]", current_node_id)?;
                self.count += 1;

                let holder_id = self.visit_expr(holder)?;
                let value_id = self.visit_expr(value)?;

                writeln!(
                    self.out,
                    "\tN_{0} -> N_{1} [label = \"holder\"]\n\tN_{0} -> N_{2} [label = \"value\"]",
                    current_node_id, holder_id, value_id
                )?;
            }

            Expr::If {
                condition,
                body,
                else_body,
            } => {
                writeln!(self.out, "\tN_{} [label = \"if\"]\n", current_node_id)?;
                self.count += 1;

                let condition_id = self.visit_expr(condition)?;
                let body = self.visit_expr(body)?;
                let else_body = self.visit_expr(else_body)?;

                writeln!(
                    self.out,
                    "\tN_{} -> N_{} [label = \"condition\"]",
                    current_node_id, condition_id
                )?;

                writeln!(
                    self.out,
                    "\tN_{} -> N_{} [label = \"truthy\"]",
                    current_node_id, body
                )?;
                writeln!(
                    self.out,
                    "\tN_{} -> N_{} [label = \"falsy\"]",
                    current_node_id, else_body
                )?;
            }
            Expr::FunctionCall { name, args } => {
                writeln!(
                    self.out,
                    "\tN_{} [label = \"func_call: {}\"]",
                    current_node_id, name
                )?;
                self.count += 1;
                for (index, node) in args.iter().enumerate() {
                    let target_id = self.visit_expr(node)?;
                    writeln!(
                        self.out,
                        "\tN_{} -> N_{} [label = \"{}\"]",
                        current_node_id, target_id, index
                    )?;
                }
            }
        }
        Ok(current_node_id)
    }
}
