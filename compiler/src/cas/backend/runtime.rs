use std::cmp::Ordering;
use std::collections::VecDeque;
use std::{collections::HashMap, ops::Deref};

use matex_common::node::{BinOp, Expr, Statement};

use matex_common::node::Visitor;

use super::format::ValueFormatter;

#[derive(Default)]
pub struct Runtime {
    pub functions: HashMap<String, Expr>,
    pub variables: HashMap<String, RuntimeVal>,
}

impl Runtime {
    pub fn run(&mut self, program: &Statement) -> RuntimeVal {
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

#[derive(Clone, Debug, PartialEq)]
// Better name
pub enum RuntimeVal {
    Unit,

    // TODO: Add complex, real, etc
    Number(f64),
    Symbol(String),

    Bool(bool),

    Sum(Terms),
    Product(Factors),
    Exponent(Box<RuntimeVal>, Box<RuntimeVal>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Factors(pub VecDeque<RuntimeVal>);
#[derive(Clone, Debug, PartialEq)]
pub struct Terms(pub VecDeque<RuntimeVal>);

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

            (Sum(Terms(v)), _) => {
                v.push_back(other);
                return self;
            }

            (_, Sum(Terms(v))) => {
                v.push_back(self);
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
                return RuntimeVal::Sum(Terms(VecDeque::from([self, other])));
            }
        }
    }
    fn multiply(mut self, mut other: RuntimeVal) -> RuntimeVal {
        use RuntimeVal::*;
        match (&mut self, &mut other) {
            (Unit, _) | (_, Unit) => panic!("Unit error when multiplicating"),

            (Bool(_), _) | (_, Bool(_)) => panic!("No multiplication with booleans"),

            (Number(lhs), Number(rhs)) => Number(lhs.deref() * rhs.deref()),

            (Product(Factors(v)), _) => {
                v.push_back(other);
                return self;
            }

            (_, Product(Factors(v))) => {
                v.push_back(self);
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
                return RuntimeVal::Product(Factors(VecDeque::from([self, other])));
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
        use RuntimeVal::*;
        match self {
            Sum(terms) => {
                RuntimeVal::combine_like_terms(terms);
                dbg!(&terms);
                RuntimeVal::combine_integers(terms);
                dbg!(&terms);
                RuntimeVal::rearrange(terms);
                dbg!(&terms);
            }
            Product(factors) => {
                let coeff = RuntimeVal::extract_coefficient(factors);

                let Factors(factors) = factors;

                if coeff != 1.0 {
                    factors.push_front(coeff.into());
                }
            }
            _ => {}
        }
        self.flatten();
        dbg!(&self);
    }

    fn combine_like_terms(terms: &mut Terms) {
        // Extract the coefficients from each term
        let mut term_coefficients = RuntimeVal::extract_coefficients(terms);

        dbg!(&term_coefficients);
        dbg!(&terms);

        let mut new_terms: VecDeque<RuntimeVal> = VecDeque::new();

        while let Some((co_eff, term)) = term_coefficients.pop() {
            let mut coefficient_total = co_eff;

            let mut i = 0;
            while i < term_coefficients.len() {
                let (co_eff, other_term) = &term_coefficients[i];

                if term.struct_equal(other_term) {
                    coefficient_total += co_eff.deref();
                    term_coefficients.remove(i);
                } else {
                    i += 1;
                }
            }


            if coefficient_total == 1.0 {
                new_terms.push_front(term);
            } else if coefficient_total == 0.0 {
                new_terms.push_back(RuntimeVal::Number(0.0));
            } else {
                let term =
                    RuntimeVal::Product(Factors(VecDeque::from([coefficient_total.into(), term])));

                if coefficient_total.is_sign_positive() {
                    new_terms.push_front(term);
                } else {
                    new_terms.push_back(term);
                }
            }
            dbg!(&new_terms);
        }
        *terms = Terms(new_terms);
    }

    fn combine_integers(Terms(terms): &mut Terms) {
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

        if total == 0.0 {
            return;
        }

        let constant = RuntimeVal::Number(total);

        terms.push_back(constant);
    }

    fn rearrange(Terms(terms): &mut Terms) {
        use RuntimeVal::*;
        let terms = terms.make_contiguous();
        terms.sort_by(|a, b| {
            dbg!(&a);
            dbg!(&b);
            return match (a, b) {
                (_, Product(factors)) => {
                    return if RuntimeVal::product_is_negative(&factors) {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }
                (Product(factors), _) if RuntimeVal::product_is_negative(&factors) => Ordering::Greater,
                (_, _) => Ordering::Less
            };
        });
        dbg!(&terms);
    }

    fn flatten(&mut self) {
        use RuntimeVal::*;
        match self {
            Sum(Terms(v)) | Product(Factors(v)) if v.len() == 1 => {
                let value = &mut v[0];
                value.flatten();
                *self = value.clone();
            }
            _ => {}
        }
    }

    fn struct_equal(&self, other: &RuntimeVal) -> bool {
        match (self, other) {
            (RuntimeVal::Sum(Terms(terms)), RuntimeVal::Sum(Terms(other))) => {
                if terms.len() != other.len() {
                    return false;
                }

                // CODE HERE TO VERIFY THAT the vectors `terms` and `other` are structurally equal:
                let mut terms_remaining = other.iter().collect::<Vec<_>>();

                for term in terms {
                    let mut found_match = false;
                    for i in 0..terms_remaining.len() {
                        let other_term = &terms_remaining[i];
                        if term.struct_equal(other_term) {
                            found_match = true;
                            terms_remaining.remove(i);
                            break;
                        }
                    }

                    return found_match;
                }

                eprintln!("Sum(terms) was empty?!");
                true
            }
            (RuntimeVal::Product(Factors(factors)), RuntimeVal::Product(Factors(other))) => {
                if factors.len() != other.len() {
                    return false;
                }

                // Verify that the vectors `factors` and `other` are structurally equal:
                let mut factors_remaining = other.iter().collect::<Vec<_>>();

                for factor in factors {
                    let mut found_match = false;
                    for i in 0..factors_remaining.len() {
                        let other_term = &factors_remaining[i];
                        if factor.struct_equal(other_term) {
                            found_match = true;
                            factors_remaining.remove(i);
                            break;
                        }
                    }

                    return found_match;
                }

                true
            }
            (RuntimeVal::Exponent(base, exp), RuntimeVal::Exponent(other_base, other_exp)) => {
                return base.struct_equal(other_base) && exp.struct_equal(other_exp)
            }
            (RuntimeVal::Number(num), RuntimeVal::Number(other)) => num == other,
            (RuntimeVal::Symbol(symbol), RuntimeVal::Symbol(other)) => symbol == other,
            _ => false,
        }
    }

    fn extract_coefficients(Terms(terms): &mut Terms) -> Vec<(f64, RuntimeVal)> {
        let mut term_coefficients: Vec<(f64, RuntimeVal)> = Vec::new();

        for term in terms {
            dbg!(&term);

            let coeff = if let RuntimeVal::Product(ref mut factors) = term {
                RuntimeVal::extract_coefficient(factors)
            } else {
                1.0
            };

            term.simplify();
            term_coefficients.push((coeff, term.clone()));
            dbg!(&term_coefficients);
        }
        term_coefficients
    }

    fn extract_coefficient(Factors(factors): &mut Factors) -> f64 {
        let mut index = 0;
        let mut coeff = 1.0;

        while index < factors.len() {
            let factor = &factors[index];

            dbg!(&factor);

            if let RuntimeVal::Number(n) = factor {
                coeff *= *n;
                dbg!(&coeff);
                dbg!(&factor);
                factors.remove(index);
            } else {
                index += 1;
            }
        }
        dbg!(&coeff);
        coeff
    }

    fn product_is_negative(Factors(factors): &Factors) -> bool {
        use RuntimeVal::*;

        factors.iter().fold(false, |is_negative, factor| {
            if let Number(n) = factor {
                if n.is_sign_negative() {
                    return !is_negative
                }
            }
            is_negative
        })
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
