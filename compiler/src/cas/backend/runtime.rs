use matex_common::function::{Function, Parameter};
use matex_common::node::{BinOp, Expr, Program, Statement};

use matex_common::node::Visitor;

use log::{debug, error};

use crate::cas::backend::value::{Terms, Factors};

use super::environment::{Environment, Scope};
use super::value::{RunType, RunVal};

macro_rules! runtime_debug {
    ($($arg:tt)+) => (debug!(target: "matex::runtime", "[{}:{}] {}", file!(), line!(), &format!($($arg)+)));
}

macro_rules! runtime_error {
    ($($arg:tt)+) => (error!(target: "matex::runtime", "[{}:{}] {}", file!(), line!(), &format!($($arg)+)));
}
pub struct Runtime {
    pub environment: Environment,
    assign: bool,
    in_func_call: bool,
}

impl Runtime {
    pub fn run(&mut self, program: &Program) -> RunVal {
        self.visit_program(program)
    }

    pub fn new() -> Self {
        Self {
            environment: Environment {
                scopes: vec![Scope::default()],
                ..Default::default()
            },
            assign: false,
            in_func_call: false,
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

        self.environment.set_func(func_name, function);

        RunType::Unit.into()
    }

    fn visit_unset_variable(&mut self, name: &String) -> RunVal {
        runtime_debug!("Visit unset variable");
        runtime_debug!("name: {}", name);

        self.environment.remove_variable(name);

        RunType::Unit.into()
    }

    fn visit_variable(&mut self, name: &String) -> RunVal {
        runtime_debug!("Visit variable");
        runtime_debug!("name: {}", name);

        if self.assign {
            return RunType::Symbol(name.clone()).into()
        }

        if let Some(mut value) = self.environment.get_variable(name).cloned() {
            runtime_debug!("value of variable: {:?}", value);
            if !self.in_func_call {
                self.get_reactive_value(&mut value); 
            }
            value
        } else {
            RunType::Symbol(name.clone()).into()
        }
    }

    // TODO: Currently only (-) unary
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

            BinOp::Equal => lhs.equal(rhs),

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

        self.assign = true;
        let value = self.visit_expr(value);
        self.assign = false;

        runtime_debug!("\n\tholder: {:?}\n\tvalue: {:?}", holder, value);

        self.environment.set_variable(holder, value.clone());

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

        if let Some(intrinsic) = self.environment.get_intrinsic(name).cloned() {
            let prev_assign = self.assign;
            self.assign = false;
            let args: Vec<RunVal> = arguments.iter().map(|it| self.visit_expr(it)).collect();
            self.assign = prev_assign;
            return intrinsic(&args);
        }

        let Some(Function { name: _, params, body }) = self.environment.get_function(name).cloned() else {
            let mut vec = Vec::new();
            for argument in arguments {
                let value = self.visit_expr(argument);
                vec.push(value);
            } 
            return RunVal::new(RunType::Function(name.clone(), vec));
        };


        // TODO: Make this better? Utilise environment.set_variable?
        let mut new_scope = Scope::default();

        for (arg_value, param) in arguments.iter().zip(params) {
            let val = self.visit_expr(arg_value);
            new_scope.variables.insert(param.name.clone(), val);
        }

        runtime_debug!("Function arguments passed");
        runtime_debug!("vars: {:?}", new_scope.variables);

        self.environment.push_scope(new_scope);

        let prev_assign = self.assign;
        self.assign = false;

        self.in_func_call = true;
        let value = self.visit_expr(&body);
        self.in_func_call = false;

        self.assign = prev_assign;

        // Reset the scope again???
        self.environment.pop_scope();

        value
    }
}

impl Runtime {
    fn get_reactive_value(&mut self, value: &mut RunVal) {
        use RunType::*; 
        runtime_debug!("value reactive: {:?}", value);
        value.simplified = false;

        match &mut value.typ {
            Symbol(sym) => {
                if let Some(mut variable_value) = self.environment.get_variable(sym).cloned() {
                    runtime_debug!("variable_value: {:?}", variable_value);
                    if let RunType::Symbol(replaced_symbol) = &variable_value.typ {
                        if sym == replaced_symbol {
                            return;
                        }
                    }
                    if false {
                        self.get_reactive_value(&mut variable_value);
                    }
                    *value = variable_value;
                }
            }

            Product(Factors(vec)) 
            | Sum(Terms(vec)) => {
                for item in vec {
                    self.get_reactive_value(item);
                }
            }
            Exponent(base, exp) => {
                self.get_reactive_value(base);
                self.get_reactive_value(exp);
            }
            Function(_, _) => todo!(),

            Unit 
            | Undefined 
            | Number(_) 
            | Bool(_) => {}
        }
    }
}

impl Visitor<RunVal> for Runtime {
    // TODO: Take ownership instead, since statements are never visited twice
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
        let mut value = match expr {
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
        };
        value.simplify();
        value
    }
}
