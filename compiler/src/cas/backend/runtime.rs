use std::{collections::HashMap, ops::Deref};

use matex_common::node::{BinOp, Expr, Statement};

use matex_common::node::Visitor;

use super::format::ValueFormatter;

#[derive(Default)]
pub struct Runtime {
    functions: HashMap<String, Expr>,
    variables: HashMap<String, RuntimeVal>,
}

impl Runtime {
    pub fn run(&mut self, program: Statement) -> RuntimeVal {
        let Statement::Program(v) = program else {
            return RuntimeVal::Unit;
        };
        self.visit_program(&v)
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

impl Visitor<RuntimeVal> for Runtime {
    fn visit_statement(&mut self, statement: &Statement) -> RuntimeVal {
        match statement {
            Statement::Program(statements) => self.visit_program(statements),
            Statement::Function {
                name,
                body: function_body,
            } => self.visit_function(name, function_body),
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
                dbg!(&expr);

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

#[derive(Clone, Debug)]
// Better name
pub enum RuntimeVal {
    Unit,

    // TODO: Add complex, real, etc
    Number(f64),
    Symbol(String),

    Bool(bool),

    Sum(Vec<RuntimeVal>),
    Product(Vec<RuntimeVal>),
    Exponent(Box<RuntimeVal>, Box<RuntimeVal>),
}

impl RuntimeVal {
    pub fn format<T: ValueFormatter>(&self) {
        T::format(self);
    }

    fn add(mut self, mut other: RuntimeVal) -> RuntimeVal {
        use RuntimeVal::*;
        match (&mut self, &mut other) {
            (Unit, _) | (_, Unit) => panic!("Unit error when adding"),

            (Bool(_), _) | (_, Bool(_)) => panic!("No addition with booleans"),

            (Number(lhs), Number(rhs)) => Number(lhs.deref() + rhs.deref()),

            (Sum(v), _) => {
                v.push(other);
                return self;
            }

            (_, Sum(v)) => {
                v.push(self);
                return other;
            }

            (Number(_), Symbol(_))
            | (Number(_), Product(_))
            | (Number(_), Exponent(_, _))
            | (Symbol(_), Number(_))
            | (Symbol(_), Symbol(_))
            | (Symbol(_), Product(_))
            | (Symbol(_), Exponent(_, _))
            | (Product(_), Number(_))
            | (Product(_), Symbol(_))
            | (Product(_), Product(_))
            | (Product(_), Exponent(_, _))
            | (Exponent(_, _), Number(_))
            | (Exponent(_, _), Symbol(_))
            | (Exponent(_, _), Product(_))
            | (Exponent(_, _), Exponent(_, _)) => {
                return RuntimeVal::Sum(vec![self, other]);
            }
        }
    }
    fn multiply(mut self, mut other: RuntimeVal) -> RuntimeVal {
        use RuntimeVal::*;
        match (&mut self, &mut other) {
            (Unit, _) | (_, Unit) => panic!("Unit error when multiplicating"),

            (Bool(_), _) | (_, Bool(_)) => panic!("No multiplication with booleans"),

            (Number(lhs), Number(rhs)) => Number(lhs.deref() * rhs.deref()),

            (Product(v), _) => {
                v.push(other);
                return self;
            }

            (_, Product(v)) => {
                v.push(self);
                return other;
            }

            (Number(_), Symbol(_))
            | (Number(_), Sum(_))
            | (Number(_), Exponent(_, _))
            | (Symbol(_), Number(_))
            | (Symbol(_), Symbol(_))
            | (Symbol(_), Sum(_))
            | (Symbol(_), Exponent(_, _))
            | (Sum(_), Number(_))
            | (Sum(_), Symbol(_))
            | (Sum(_), Sum(_))
            | (Sum(_), Exponent(_, _))
            | (Exponent(_, _), Number(_))
            | (Exponent(_, _), Symbol(_))
            | (Exponent(_, _), Sum(_))
            | (Exponent(_, _), Exponent(_, _)) => {
                return RuntimeVal::Product(vec![self, other]);
            }
        }
    }
    fn power(self, other: RuntimeVal) -> RuntimeVal {
        use RuntimeVal::*;
        match (&self, &other) {
            (Unit, _) | (_, Unit) => panic!("Unit error when powering"),

            (Bool(_), _) | (_, Bool(_)) => panic!("No powering with booleans"),

            (Number(lhs), Number(rhs)) => Number(lhs.powf(*rhs)),

            (Number(_), Symbol(_))
            | (Number(_), Sum(_))
            | (Number(_), Product(_))
            | (Number(_), Exponent(_, _))
            | (Symbol(_), Number(_))
            | (Symbol(_), Symbol(_))
            | (Symbol(_), Sum(_))
            | (Symbol(_), Product(_))
            | (Symbol(_), Exponent(_, _))
            | (Sum(_), Number(_))
            | (Sum(_), Symbol(_))
            | (Sum(_), Sum(_))
            | (Sum(_), Product(_))
            | (Sum(_), Exponent(_, _))
            | (Product(_), Number(_))
            | (Product(_), Symbol(_))
            | (Product(_), Sum(_))
            | (Product(_), Product(_))
            | (Product(_), Exponent(_, _))
            | (Exponent(_, _), Number(_))
            | (Exponent(_, _), Symbol(_))
            | (Exponent(_, _), Sum(_))
            | (Exponent(_, _), Product(_))
            | (Exponent(_, _), Exponent(_, _)) => {
                return Exponent(Box::new(self), Box::new(other));
            }
        }
    }
    fn less(self, other: RuntimeVal) -> RuntimeVal {
        match (&self, &other) {
            (RuntimeVal::Number(lhs), RuntimeVal::Number(rhs)) => RuntimeVal::Bool(lhs < rhs),

            _ => {
                todo!("Not implemented: {:?} {:?}", self, other);
            }
        }
    }
    fn less_equal(self, other: RuntimeVal) -> RuntimeVal {
        match (&self, &other) {
            (RuntimeVal::Number(lhs), RuntimeVal::Number(rhs)) => RuntimeVal::Bool(lhs <= rhs),

            _ => {
                todo!("Not implemented: {:?} {:?}", self, other);
            }
        }
    }
    fn greater(self, other: RuntimeVal) -> RuntimeVal {
        match (&self, &other) {
            (RuntimeVal::Number(lhs), RuntimeVal::Number(rhs)) => RuntimeVal::Bool(lhs > rhs),

            _ => {
                todo!("Not implemented: {:?} {:?}", self, other);
            }
        }
    }
    fn greater_equal(self, other: RuntimeVal) -> RuntimeVal {
        match (&self, &other) {
            (RuntimeVal::Number(lhs), RuntimeVal::Number(rhs)) => RuntimeVal::Bool(lhs >= rhs),

            _ => {
                todo!("Not implemented: {:?} {:?}", self, other);
            }
        }
    }
}

impl RuntimeVal {
    fn simplify(&mut self) {
        /*let test = self.struct_equal(&mut AlgebraExpr::Sum(vec![
            AlgebraExpr::Product(vec![(3.0).into(), AlgebraExpr::Symbol("x".to_owned())]),
            AlgebraExpr::Product(vec![(2.0).into(), AlgebraExpr::Symbol("x".to_owned())]),
        ]));

        dbg!(test);
        */

        self.combine_like_terms();
        dbg!(&self);
        self.combine_integers();
        dbg!(&self);
    }

    fn combine_integers(&mut self) {
        dbg!(&self);
        match self {
            RuntimeVal::Sum(terms) => {
                if terms.len() == 1 {
                    *self = terms[0].clone();
                    return;
                }

                let mut total = 0.0;

                let mut i = 0;
                while i < terms.len() {
                    if let RuntimeVal::Number(n) = terms[i] {
                        total += n;
                        terms.remove(i);
                    } else {
                        i += 1;
                    }
                }

                let constant = RuntimeVal::Number(total);
                if terms.is_empty() {
                    *self = constant
                } else if total != 0.0 {
                    terms.push(constant);
                }
            }
            RuntimeVal::Product(factors) => {
                if factors.len() == 1 {
                    *self = factors[0].clone();
                    return;
                }

                let mut total = 1.0;

                let mut i = 0;
                while i < factors.len() {
                    if let RuntimeVal::Number(n) = factors[i] {
                        total *= n;
                        factors.remove(i);
                    } else {
                        i += 1;
                    }
                }

                dbg!(&total);
                if total != 1.0 {
                    dbg!(&total);
                    factors.push(RuntimeVal::Number(total));
                }
            }
            RuntimeVal::Exponent(_, _) => todo!(),
            RuntimeVal::Number(_) | RuntimeVal::Symbol(_) => {}
            RuntimeVal::Unit => todo!(),
            RuntimeVal::Bool(_) => todo!(),
        }
    }

    fn combine_like_terms(&mut self) {
        match self {
            Self::Sum(terms) => {
                let mut term_coefficients = Vec::new();

                // Extract the coefficients from each term
                for term in terms {
                    let mut co_efficient = 1.0;
                    let mut index = 0;
                    if let RuntimeVal::Product(factors) = term {
                        while index < factors.len() {
                            let factor = &factors[index];
                            if let RuntimeVal::Number(n) = factor {
                                co_efficient *= n;
                                factors.remove(index);
                                term.simplify();
                                break;
                            }
                            index += 1;
                        }
                    }

                    let test = (co_efficient, term.clone());
                    dbg!(&test);
                    term_coefficients.push(test);
                }

                dbg!(&term_coefficients);
                // Combine like terms
                #[allow(unused_variables, unused_mut)]
                let mut new_terms: Vec<RuntimeVal> = Vec::new();

                while let Some((co_eff, mut term)) = term_coefficients.pop() {
                    let mut coefficient_total = co_eff;

                    let mut i = 0;
                    while i < term_coefficients.len() {
                        let (co_eff, other_term) = &mut term_coefficients[i];
                        if term.struct_equal(other_term) {
                            coefficient_total += co_eff.deref();
                            term_coefficients.remove(i);
                        } else {
                            i += 1;
                        }
                    }

                    if coefficient_total == 1.0 {
                        new_terms.push(term);
                    } else {
                        let mut term = RuntimeVal::Product(vec![coefficient_total.into(), term]);
                        term.simplify();
                        new_terms.push(term);
                    }
                    dbg!(&new_terms);
                }
                *self = RuntimeVal::Sum(new_terms);
            }
            _ => {}
        }
    }

    fn struct_equal(&mut self, other: &mut RuntimeVal) -> bool {
        match (self, other) {
            (RuntimeVal::Sum(terms), RuntimeVal::Sum(other)) => {
                if terms.len() != other.len() {
                    return false;
                }

                // CODE HERE TO VERIFY THAT the vectors `terms` and `other` are structurally equal:
                let mut other_terms_remaining = other.iter_mut().collect::<Vec<_>>();

                for term in terms {
                    let mut found_match = false;
                    for i in 0..other_terms_remaining.len() {
                        let other_term = &mut other_terms_remaining[i];
                        if term.struct_equal(other_term) {
                            found_match = true;
                            other_terms_remaining.remove(i);
                            break;
                        }
                    }
                    if !found_match {
                        return false;
                    }
                }

                true
            }
            (RuntimeVal::Product(factors), RuntimeVal::Product(other)) => {
                if factors.len() != other.len() {
                    return false;
                }

                // Verify that the vectors `factors` and `other` are structurally equal:
                let mut other_terms_remaining = other.iter_mut().collect::<Vec<_>>();

                for factor in factors {
                    let mut found_match = false;
                    for i in 0..other_terms_remaining.len() {
                        let other_term = &mut other_terms_remaining[i];
                        if factor.struct_equal(other_term) {
                            found_match = true;
                            other_terms_remaining.remove(i);
                            break;
                        }
                    }
                    if !found_match {
                        return false;
                    }
                }

                true
            }
            (RuntimeVal::Exponent(_, _), RuntimeVal::Exponent(_, _)) => todo!(),
            (RuntimeVal::Number(num), RuntimeVal::Number(other)) => num == other,
            (RuntimeVal::Symbol(symbol), RuntimeVal::Symbol(other)) => symbol == other,
            _ => false,
        }
    }
}

impl Into<RuntimeVal> for String {
    fn into(self) -> RuntimeVal {
        RuntimeVal::Symbol(self)
    }
}

impl Into<RuntimeVal> for f64 {
    fn into(self) -> RuntimeVal {
        RuntimeVal::Number(self)
    }
}
