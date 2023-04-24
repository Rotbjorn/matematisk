use std::fmt::{Error, Write};

use crate::{function::Function, token::TokenType};

type Statements = Vec<Statement>;

#[derive(Debug)]
pub struct Program(pub Statements);

#[derive(Debug)]
pub enum Statement {
    FunctionDefinition(Function),
    UnsetVariable(String),
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    Variable(String),

    List(Vec<Expr>),

    // Currently only negation unary (-expr)
    Unary(Box<Expr>),

    Simplify(Box<Expr>),

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

impl From<&TokenType> for BinOp {
    fn from(value: &TokenType) -> Self {
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
    // TODO: Verify precedence, -2^3 or (-2)^3?
    Unary,
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

    pub fn create_dot_graph(&mut self, Program(statements): &Program) -> Result<(), Error> {
        self.out.write_str("digraph AST {\n")?;
        self.out.write_str("\tlabel = \"Abstract Syntax Tree\"\n")?;
        self.out.write_str("\tfontname = \"Arial\"\n")?;
        self.out.write_str("\tnode [fontname = \"Arial\"]\n")?;
        self.out.write_str("\tedge [fontname = \"Arial\"]\n\n")?;

        let current = self.count;
        self.create_node("<>")?;

        for (index, statement) in statements.iter().enumerate() {
            let node = self.visit_statement(statement)?;
            self.create_edge_label(current, node, &index.to_string())?;
        }

        self.out.write_char('}')?;

        Ok(())
    }

    fn create_node(&mut self, label: &str) -> GraphResult<u32> {
        writeln!(self.out, "\tN_{} [label = \"{}\"]", self.count, label)?;
        self.count += 1;
        Ok(self.count)
    }

    fn create_edge(&mut self, from: u32, to: u32) -> GraphResult<()> {
        writeln!(self.out, "\tN_{} -> N_{}", from, to)?;
        Ok(())
    }

    fn create_edge_label(&mut self, from: u32, to: u32, label: &str) -> GraphResult<()> {
        writeln!(self.out, "\tN_{} -> N_{} [label = \"{}\"]", from, to, label)?;
        Ok(())
    }
}

type GraphResult<T> = Result<T, Error>;
// TODO: There is a library for creating Graphviz dot files!
impl<'a, W: Write> Visitor<Result<u32, Error>> for ASTGraphGenerator<'a, W> {
    fn visit_statement(&mut self, statement: &Statement) -> Result<u32, Error> {
        let current = self.count;
        match statement {
            Statement::FunctionDefinition(Function {
                name,
                params: _,
                body,
            }) => {
                self.create_node(&format!("func: {}", name))?;

                let body = self.visit_expr(body)?;

                self.create_edge(current, body)?;
            }
            Statement::UnsetVariable(symbol) => {
                self.create_node(&format!("unset: {}", symbol))?;
            }
            Statement::Expression(expr) => {
                self.visit_expr(expr)?;
                self.count += 1;
            }
        }

        Ok(current)
    }

    fn visit_expr(&mut self, expr: &Expr) -> Result<u32, Error> {
        let current = self.count;
        match expr {
            Expr::List(v) => {
                self.create_node("<>")?;

                for (index, node) in v.iter().enumerate() {
                    let node = self.visit_expr(node)?;
                    self.create_edge_label(current, node, &index.to_string())?;
                }
            }
            Expr::Number(n) => {
                self.create_node(&format!("num: {}", n))?;
            }
            Expr::Variable(name) => {
                self.create_node(&format!("var: {}", name))?;
            }
            Expr::Unary(expr) => {
                self.create_node("unary (-)")?;

                let expr = self.visit_expr(expr)?;

                self.create_edge(current, expr)?;
            }
            Expr::Simplify(expr) => {
                self.create_node("simplify")?;

                let expr = self.visit_expr(expr)?;

                self.create_edge(current, expr)?;
            }
            Expr::BinaryOp {
                left,
                operation,
                right,
            } => {
                self.create_node(&format!("{:?}", operation))?;

                let left = self.visit_expr(left)?;
                let right = self.visit_expr(right)?;

                match *operation {
                    BinOp::Power => {
                        self.create_edge_label(current, left, "base")?;
                        self.create_edge_label(current, right, "exponent")?;
                    }
                    BinOp::Divide => {
                        self.create_edge_label(current, left, "numer")?;
                        self.create_edge_label(current, right, "denom")?;
                    }
                    _ => {
                        self.create_edge_label(current, left, "lhs")?;
                        self.create_edge_label(current, right, "rhs")?;
                    }
                }
            }

            Expr::Assignment { holder, value } => {
                self.create_node("Assignment")?;

                let holder = self.visit_expr(holder)?;
                let value = self.visit_expr(value)?;

                self.create_edge_label(current, holder, "holder")?;
                self.create_edge_label(current, value, "value")?;
            }

            Expr::If {
                condition,
                body,
                else_body,
            } => {
                self.create_node("if")?;

                let condition = self.visit_expr(condition)?;
                let body = self.visit_expr(body)?;
                let else_body = self.visit_expr(else_body)?;

                self.create_edge_label(current, condition, "condition")?;

                self.create_edge_label(current, body, "truthy")?;
                self.create_edge_label(current, else_body, "falsy")?;
            }
            Expr::FunctionCall { name, args } => {
                self.create_node(&format!("func_call: {}", name))?;

                for (index, node) in args.iter().enumerate() {
                    let target_id = self.visit_expr(node)?;
                    self.create_edge_label(current, target_id, &index.to_string())?;
                }
            }
        }
        Ok(current)
    }
}
