use std::{collections::HashMap, ops::Deref};

use matex_common::node::{BinOp, Expr, Statement};

use matex_common::node::Visitor;

#[derive(Clone, Debug)]
pub enum RuntimeVal {
    Unit,

    // TODO: Add complex, real, etc
    Number(f64),
    Symbol(String),
    Expression(Expr),

    Bool(bool),
}

#[derive(Default)]
pub struct RuntimeVisitor {
    functions: HashMap<String, Expr>,
    variables: HashMap<String, RuntimeVal>,
}

impl Visitor<RuntimeVal> for RuntimeVisitor {
    fn visit_statement(&mut self, statement: &Statement) -> RuntimeVal {
        match statement {
            Statement::Program(statements) => self.visit_program(statements),
            Statement::Function {
                name,
                function_body,
            } => self.visit_function(name, function_body),
            Statement::Expression(expr) => self.visit_expr(expr),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> RuntimeVal {
        dbg!(expr);
        match expr {
            Expr::Number(n) => RuntimeVal::Number(*n),
            Expr::Variable(name) => self.visit_variable(name),
            Expr::List(_) => todo!("Not handling List"),
            Expr::BinaryOp {
                left,
                operation,
                right,
            } => self.visit_binary_operation(left, operation, right),
            Expr::Assignment { holder, value } => self.visit_assignment(holder, value),
            Expr::If {
                condition,
                body,
                else_body,
            } => self.visit_if(condition, body, else_body),
            Expr::FunctionCall { name, args } => self.visit_function_call(name, args),
        }
    }
}

impl RuntimeVisitor {
    fn visit_program(&mut self, statements: &Vec<Statement>) -> RuntimeVal {
        let mut value = RuntimeVal::Number(-1.0);
        for statement in statements {
            value = self.visit_statement(statement);
        }

        value
    }

    fn visit_function(&mut self, func_name: &str, function_body: &Expr) -> RuntimeVal {
        // TODO: Move ownership instead of clone?
        self.functions
            .insert(func_name.to_owned(), function_body.clone());

        RuntimeVal::Unit
    }

    fn visit_variable(&mut self, name: &String) -> RuntimeVal {
        if let Some(variable) = self.variables.get(name) {
            variable.clone()
        } else {
            RuntimeVal::Symbol(name.clone())
        }
    }

    fn visit_binary_operation(
        &mut self,
        left: &Expr,
        operation: &BinOp,
        right: &Expr,
    ) -> RuntimeVal {
        let lhs = self.visit_expr(left);
        let rhs = self.visit_expr(right);

        match lhs {
            RuntimeVal::Number(lhs) => match rhs {
                RuntimeVal::Number(rhs) => match operation {
                    BinOp::Add => RuntimeVal::Number(lhs + rhs),
                    BinOp::Subtract => RuntimeVal::Number(lhs - rhs),
                    BinOp::Multiply => RuntimeVal::Number(lhs * rhs),
                    BinOp::Divide => RuntimeVal::Number(lhs / rhs),
                    BinOp::Power => RuntimeVal::Number(lhs.powf(rhs)),
                    BinOp::Less => RuntimeVal::Bool(lhs < rhs),
                    BinOp::LessEqual => RuntimeVal::Bool(lhs <= rhs),
                    BinOp::Greater => RuntimeVal::Bool(lhs > rhs),
                    BinOp::GreaterEqual => RuntimeVal::Bool(lhs >= rhs),
                    _ => panic!("Unhandled binary operation"),
                },
                _ => todo!("Error"),
            },
            _ => todo!("Error"),
        }
    }

    fn visit_assignment(&mut self, holder: &Expr, value: &Expr) -> RuntimeVal {
        dbg!(holder, value);

        // TOOD: Move ownership instead?
        let Expr::Variable(holder) = holder.deref() else {
            panic!("Unhandled holder expression");
        };

        let value = self.visit_expr(value);

        dbg!(&holder, &value);

        self.variables.insert(holder.clone(), value.clone());

        value
    }

    fn visit_if(&mut self, condition: &Expr, body: &Expr, else_body: &Expr) -> RuntimeVal {
        let condition = self.visit_expr(condition);

        let RuntimeVal::Bool(b) = condition else {
            eprintln!("Expected a boolean value, got {:?}", condition);
            return RuntimeVal::Unit
        };

        let value = if b {
            self.visit_expr(body)
        } else {
            self.visit_expr(else_body)
        };

        value
    }

    fn visit_function_call(&mut self, name: &String, arguments: &Vec<Expr>) -> RuntimeVal {
        let mut runtime_vals = Vec::<RuntimeVal>::new();

        for argument in arguments {
            let val = self.visit_expr(argument);
            runtime_vals.push(val)
        }

        let Some(expr) = self.functions.get(name) else {
            return RuntimeVal::Unit;
        };

        let value = self.visit_expr(&expr.clone());

        value
    }
}
