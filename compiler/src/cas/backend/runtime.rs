use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use matex_common::function::{Function, Parameter};
use matex_common::node::{BinOp, Expr, Statement};

use matex_common::node::Visitor;

use super::environment::{Environment, Scope};
use super::value::RuntimeVal;

pub struct Runtime {
    pub environment: Environment,
}

impl Runtime {
    pub fn run(&mut self, program: &Statement) -> RuntimeVal {
        let Statement::Program(v) = program else {
            return RuntimeVal::Unit;
        };
        self.visit_program(&v)
    }

    pub fn new() -> Self {
        Self {
            environment: Environment {
                scopes: vec![Rc::new(RefCell::new(Scope::default()))],
            },
        }
    }
}

impl Runtime {
    fn visit_program(&mut self, statements: &Vec<Statement>) -> RuntimeVal {
        let mut value = RuntimeVal::Number(-1.0);
        for statement in statements {
            value = self.visit_statement(statement);
        }

        value
    }

    fn visit_function(
        &mut self,
        _func_name: &str,
        _params: &Vec<Parameter>,
        _function_body: &Expr,
    ) -> RuntimeVal {
        // TODO: Just use functions from Parser?
        /*self.environment.borrow_mut().functions
            .insert(func_name.to_owned(), function_body.clone());
        */
        RuntimeVal::Unit
    }

    fn visit_variable(&mut self, name: &String) -> RuntimeVal {
        if let Some(variable) = self.environment.get_scope().borrow().variables.get(name) {
            variable.clone()
        } else {
            RuntimeVal::Symbol(name.clone())
        }
    }

    fn visit_unary_operation(&mut self, expr: &Expr) -> RuntimeVal {
        let expr = self.visit_expr(expr);

        let value = expr.multiply(RuntimeVal::Number(-1.0));

        value
    }

    fn visit_binary_operation(
        &mut self,
        left: &Expr,
        operation: &BinOp,
        right: &Expr,
    ) -> RuntimeVal {
        let lhs = self.visit_expr(left);
        let rhs = self.visit_expr(right);

        let value = match *operation {
            BinOp::Add => lhs.add(rhs),
            BinOp::Subtract => {
                let rhs = rhs.multiply(RuntimeVal::Number(-1.0));
                lhs.add(rhs)
            }
            BinOp::Multiply => lhs.multiply(rhs),
            BinOp::Divide => {
                let rhs = rhs.power(RuntimeVal::Number(-1.0));
                lhs.multiply(rhs)
            }
            BinOp::Power => lhs.power(rhs),

            BinOp::Less => lhs.less(rhs),
            BinOp::LessEqual => lhs.less_equal(rhs),
            BinOp::Greater => lhs.greater(rhs),
            BinOp::GreaterEqual => lhs.greater_equal(rhs),

            _ => panic!("Not a valid binary operation!!!"),
        };

        // dbg!(&value);

        value
    }

    fn visit_assignment(&mut self, holder: &Expr, value: &Expr) -> RuntimeVal {
        // TODO: Move ownership instead?
        let Expr::Variable(holder) = holder.deref() else {
            panic!("Unhandled holder expression");
        };

        let value = self.visit_expr(value);

        dbg!(&holder, &value);

        self.environment
            .get_scope()
            .borrow_mut()
            .variables
            .insert(holder.clone(), value.clone());

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
        let binding = self.environment.get_scope();
        let binding = binding.borrow_mut();
        let Some(Function { name: _, params, body }) = binding.functions.get(name).clone() else {
            return RuntimeVal::Unit;
        };

        let mut new_scope = Scope::default();

        for (arg_value, param) in arguments.iter().zip(params) {
            let val = self.visit_expr(arg_value);
            new_scope.variables.insert(param.name.clone(), val);
        }

        self.environment
            .push_scope(Rc::new(RefCell::new(new_scope)));

        let value = self.visit_expr(&body);

        // Reset the scope again???
        self.environment.pop_scope();

        value
    }
}

impl Visitor<RuntimeVal> for Runtime {
    fn visit_statement(&mut self, statement: &Statement) -> RuntimeVal {
        match statement {
            Statement::Program(statements) => self.visit_program(statements),
            Statement::Function {
                name,
                parameters: params,
                body: function_body,
            } => self.visit_function(name, params, function_body),
            Statement::Expression(expr) => self.visit_expr(expr),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> RuntimeVal {
        match expr {
            Expr::Number(n) => RuntimeVal::Number(*n),
            Expr::Variable(name) => self.visit_variable(name),
            Expr::List(_) => todo!("Not handling List"),
            Expr::Unary(expr) => self.visit_unary_operation(expr),
            Expr::Simplify(expr) => {
                let mut expr = self.visit_expr(expr);

                expr.simplify();

                return expr;
            }
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
