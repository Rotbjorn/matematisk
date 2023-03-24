use matex_common::node::{Statement, Expr};

pub trait Visitor<T> {
    fn visit_statement(&mut self, n: &Statement) -> T;
    fn visit_expr(&mut self, n: &Expr) -> T;
}