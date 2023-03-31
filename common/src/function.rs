use crate::node::Expr;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Parameter {
    pub name: String,
    pub type_name: String,
}
