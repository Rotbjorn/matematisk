use matex_common::function::{Function, Parameter};
use matex_common::node::{BinOp, Expr, Program, Statement};

use matex_common::node::Visitor;

use log::{debug, error};

use super::environment::{Environment, Scope};
use super::value::{RunType, RunVal};

macro_rules! runtime_debug {
    ($($arg:tt)+) => (debug!(target: "matex::runtime", $($arg)+));
}

macro_rules! runtime_error {
    ($($arg:tt)+) => (error!(target: "matex::runtime", $($arg)+));
}
pub struct Runtime {
    pub environment: Environment,
}

impl Runtime {
    pub fn run(&mut self, program: &Program) -> RunVal {
        self.visit_program(program)
    }

    pub fn new() -> Self {
        Self {
            environment: Environment {
                scopes: vec![Scope::default()],
            },
        }
    }
}

impl Runtime {
    fn visit_program(&mut self, Program(statements): &Program) -> RunVal {
        let mut value = RunType::Number(-1.0).into();
        for statement in statements {
            value = self.visit_statement(statement);
        }

        value
    }

    fn visit_function(
        &mut self,
        func_name: &str,
        params: &Vec<Parameter>,
        function_body: &Expr,
    ) -> RunVal {
        runtime_debug!("Visit function declaration");
        runtime_debug!("func_name: {}", func_name);
        runtime_debug!("params: {:?}", params);
        runtime_debug!("body: {:?}", function_body);

        let function = Function {
            name: func_name.to_owned(),
            params: params.clone(),
            body: function_body.clone(),
        };

        self.environment
            .get_scope()
            .functions
            .insert(func_name.to_owned(), function);

        RunType::Unit.into()
    }

    fn visit_unset_variable(&mut self, name: &String) -> RunVal {
        runtime_debug!("Visit unset variable");
        runtime_debug!("name: {}", name);

        self.environment.get_scope().variables.remove(name);

        RunType::Unit.into()
    }

    fn visit_variable(&mut self, name: &String) -> RunVal {
        runtime_debug!("Visit variable");
        runtime_debug!("name: {}", name);

        if let Some(variable) = self.environment.get_scope().variables.get(name) {
            variable.clone()
        } else {
            RunType::Symbol(name.clone()).into()
        }
    }

    fn visit_unary_operation(&mut self, expr: &Expr) -> RunVal {
        runtime_debug!("Visit unary operation");
        runtime_debug!("expr: {:?}", expr);

        let expr = self.visit_expr(expr);

        let value = expr.multiply(RunType::Number(-1.0).into());

        value
    }

    fn visit_binary_operation(
        &mut self,
        left: &Expr,
        operation: &BinOp,
        right: &Expr,
    ) -> RunVal {
        runtime_debug!("Visit binary operation");
        runtime_debug!("left: {:?}", left);
        runtime_debug!("operation: {:?}", operation);
        runtime_debug!("right: {:?}", right);

        let lhs = self.visit_expr(left);
        let rhs = self.visit_expr(right);

        let value = match *operation {
            BinOp::Add => lhs.add(rhs),
            BinOp::Subtract => {
                let rhs = rhs.multiply(RunType::Number(-1.0).into());
                lhs.add(rhs)
            }
            BinOp::Multiply => lhs.multiply(rhs),
            BinOp::Divide => {
                let rhs = rhs.power(RunType::Number(-1.0).into());
                lhs.multiply(rhs)
            }
            BinOp::Power => lhs.power(rhs),

            BinOp::Less => lhs.less(rhs),
            BinOp::LessEqual => lhs.less_equal(rhs),
            BinOp::Greater => lhs.greater(rhs),
            BinOp::GreaterEqual => lhs.greater_equal(rhs),

            _ => {
                runtime_error!("Not a valid binary operation");
                panic!();
            }
        };

        // dbg!(&value);

        value
    }

    fn visit_assignment(&mut self, holder: &Expr, value: &Expr) -> RunVal {
        runtime_debug!("Visit assignment");
        runtime_debug!("holder: {:?}", holder);
        runtime_debug!("value: {:?}", value);
        // TODO: Move ownership instead?
        let Expr::Variable(holder) = holder else {
            panic!("Unhandled holder expression");
        };

        let value = self.visit_expr(value);

        dbg!(&holder, &value);

        self.environment
            .get_scope()
            .variables
            .insert(holder.clone(), value.clone());

        value
    }

    fn visit_if(&mut self, condition: &Expr, body: &Expr, else_body: &Expr) -> RunVal {
        runtime_debug!("Visit if");
        runtime_debug!("condition: {:?}", condition);
        runtime_debug!("body: {:?}", body);
        runtime_debug!("else: {:?}", else_body);

        let condition = self.visit_expr(condition);

        let RunType::Bool(b) = condition.typ else {
            eprintln!("Expected a boolean value, got {:?}", condition);
            return RunType::Unit.into()
        };

        let value = if b {
            self.visit_expr(body)
        } else {
            self.visit_expr(else_body)
        };

        value
    }

    fn visit_function_call(&mut self, name: &String, arguments: &Vec<Expr>) -> RunVal {
        runtime_debug!("Visit function call");
        runtime_debug!("func_name: {}", name);
        runtime_debug!("func_args: {:?}", arguments);

        let Some(Function { name: _, params, body }) = self.environment.get_scope().functions.get(name).cloned() else {
            return RunType::Unit.into();
        };

        let mut new_scope = Scope::default();

        for (arg_value, param) in arguments.iter().zip(params) {
            let val = self.visit_expr(arg_value);
            new_scope.variables.insert(param.name.clone(), val);
        }

        runtime_debug!("Function arguments passed");
        runtime_debug!("vars: {:?}", new_scope.variables);

        self.environment.push_scope(new_scope);

        let value = self.visit_expr(&body);

        // Reset the scope again???
        self.environment.pop_scope();

        value
    }
}

impl Visitor<RunVal> for Runtime {
    // TODO: Take ownership instead, since statements are never visited twice
    // Make sure Program is its own type, and not a statement.
    fn visit_statement(&mut self, statement: &Statement) -> RunVal {
        use Statement::*;
        match statement {
            FunctionDefinition(Function {
                name,
                params,
                body: function_body,
            }) => self.visit_function(name, params, function_body),
            UnsetVariable(symbol) => self.visit_unset_variable(symbol),
            Expression(expr) => self.visit_expr(expr),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> RunVal {
        match expr {
            Expr::Number(n) => RunType::Number(*n).into(),
            Expr::Variable(name) => self.visit_variable(name),
            Expr::List(_) => todo!("Not handling List"),
            Expr::Unary(expr) => self.visit_unary_operation(expr),
            Expr::Simplify(expr) => {
                let mut expr = self.visit_expr(expr);

                expr.simplify();

                expr
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
