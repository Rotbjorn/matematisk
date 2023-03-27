use std::{collections::HashMap, ops::Deref};

use matex_common::node::{BinOp, Expr, Statement};

use matex_common::node::Visitor;

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
            RuntimeVal::Expression(name.clone().into())
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
                let expr = self.visit_expr(expr);

                match expr {
                    RuntimeVal::Expression(mut expr) => {
                        expr.simplify();
                        dbg!(&expr);
                        return expr.into();
                    }
                    _ => panic!("Can't simplify non expression"),
                }
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
    Bool(bool),

    Expression(AlgebraExpr),
}

impl RuntimeVal {
    fn add(self, other: RuntimeVal) -> RuntimeVal {
        match (self, other) {
            (RuntimeVal::Number(lhs), RuntimeVal::Number(rhs)) => RuntimeVal::Number(lhs + rhs),
            (RuntimeVal::Number(lhs), RuntimeVal::Expression(mut rhs))
            | (RuntimeVal::Expression(mut rhs), RuntimeVal::Number(lhs)) => match &mut rhs {
                AlgebraExpr::Sum(v) => {
                    v.push(lhs.into());
                    return rhs.into();
                }
                AlgebraExpr::Product(_) | AlgebraExpr::Exponent(_, _) | AlgebraExpr::Symbol(_) => {
                    AlgebraExpr::Sum(vec![lhs.into(), rhs]).into()
                }

                AlgebraExpr::Number(_) => todo!("If this happens I don't know..."),
            },
            (RuntimeVal::Number(_), RuntimeVal::Bool(_)) => todo!(),

            (RuntimeVal::Expression(mut lhs), RuntimeVal::Expression(mut rhs)) => {
                match (&mut lhs, &mut rhs) {
                    (AlgebraExpr::Sum(lhs_v), AlgebraExpr::Sum(rhs_v)) => {
                        lhs_v.append(rhs_v);
                        return lhs.into();
                    }

                    (AlgebraExpr::Sum(lhs_v), AlgebraExpr::Product(_))
                    | (AlgebraExpr::Sum(lhs_v), AlgebraExpr::Exponent(_, _))
                    | (AlgebraExpr::Sum(lhs_v), AlgebraExpr::Number(_))
                    | (AlgebraExpr::Sum(lhs_v), AlgebraExpr::Symbol(_)) => {
                        lhs_v.push(rhs);
                        return lhs.into();
                    }

                    (AlgebraExpr::Product(_), AlgebraExpr::Sum(rhs_v))
                    | (AlgebraExpr::Number(_), AlgebraExpr::Sum(rhs_v))
                    | (AlgebraExpr::Symbol(_), AlgebraExpr::Sum(rhs_v))
                    | (AlgebraExpr::Exponent(_, _), AlgebraExpr::Sum(rhs_v)) => {
                        rhs_v.push(lhs);
                        return rhs.into();
                    }

                    _ => {
                        return AlgebraExpr::Sum(vec![lhs, rhs]).into();
                    }
                }
            }

            (RuntimeVal::Expression(_), RuntimeVal::Bool(_)) => todo!(),
            (RuntimeVal::Unit, RuntimeVal::Unit) => todo!(),
            (RuntimeVal::Unit, RuntimeVal::Number(_)) => todo!(),
            (RuntimeVal::Unit, RuntimeVal::Expression(_)) => todo!(),
            (RuntimeVal::Unit, RuntimeVal::Bool(_)) => todo!(),
            (RuntimeVal::Number(_), RuntimeVal::Unit) => todo!(),
            (RuntimeVal::Expression(_), RuntimeVal::Unit) => todo!(),
            (RuntimeVal::Bool(_), RuntimeVal::Unit) => todo!(),
            (RuntimeVal::Bool(_), RuntimeVal::Number(_)) => todo!(),
            (RuntimeVal::Bool(_), RuntimeVal::Expression(_)) => todo!(),
            (RuntimeVal::Bool(_), RuntimeVal::Bool(_)) => todo!(),
        }
    }
    fn multiply(self, other: RuntimeVal) -> RuntimeVal {
        match (self, other) {
            (RuntimeVal::Number(lhs), RuntimeVal::Number(rhs)) => RuntimeVal::Number(lhs * rhs),

            (RuntimeVal::Number(lhs), RuntimeVal::Expression(mut rhs))
            | (RuntimeVal::Expression(mut rhs), RuntimeVal::Number(lhs)) => match &mut rhs {
                AlgebraExpr::Product(v) => {
                    v.push(lhs.into());
                    return rhs.into();
                }

                AlgebraExpr::Sum(_) | AlgebraExpr::Exponent(_, _) | AlgebraExpr::Symbol(_) => {
                    AlgebraExpr::Product(vec![lhs.into(), rhs]).into()
                }

                AlgebraExpr::Number(_) => todo!("Shouldn't be able to happen?"),
            },
            (RuntimeVal::Number(_), RuntimeVal::Bool(_)) => todo!(),

            (RuntimeVal::Expression(mut lhs), RuntimeVal::Expression(mut rhs)) => {
                match (&mut lhs, &mut rhs) {
                    (AlgebraExpr::Product(lhs_v), AlgebraExpr::Product(rhs_v)) => {
                        lhs_v.append(rhs_v)
                    }

                    (AlgebraExpr::Product(lhs_v), AlgebraExpr::Sum(_))
                    | (AlgebraExpr::Product(lhs_v), AlgebraExpr::Exponent(_, _))
                    | (AlgebraExpr::Product(lhs_v), AlgebraExpr::Number(_))
                    | (AlgebraExpr::Product(lhs_v), AlgebraExpr::Symbol(_)) => {
                        lhs_v.push(rhs);
                        return lhs.into();
                    }

                    (AlgebraExpr::Sum(_), AlgebraExpr::Product(rhs_v))
                    | (AlgebraExpr::Number(_), AlgebraExpr::Product(rhs_v))
                    | (AlgebraExpr::Symbol(_), AlgebraExpr::Product(rhs_v))
                    | (AlgebraExpr::Exponent(_, _), AlgebraExpr::Product(rhs_v)) => {
                        rhs_v.push(lhs);
                        return rhs.into();
                    }

                    _ => {
                        return AlgebraExpr::Product(vec![lhs, rhs]).into();
                    }
                }
                panic!("UH OH!!!!");
            }

            (RuntimeVal::Expression(_), RuntimeVal::Bool(_)) => todo!(),
            (RuntimeVal::Unit, RuntimeVal::Unit) => todo!(),
            (RuntimeVal::Unit, RuntimeVal::Number(_)) => todo!(),
            (RuntimeVal::Unit, RuntimeVal::Expression(_)) => todo!(),
            (RuntimeVal::Unit, RuntimeVal::Bool(_)) => todo!(),
            (RuntimeVal::Number(_), RuntimeVal::Unit) => todo!(),
            (RuntimeVal::Expression(_), RuntimeVal::Unit) => todo!(),
            (RuntimeVal::Bool(_), RuntimeVal::Unit) => todo!(),
            (RuntimeVal::Bool(_), RuntimeVal::Number(_)) => todo!(),
            (RuntimeVal::Bool(_), RuntimeVal::Expression(_)) => todo!(),
            (RuntimeVal::Bool(_), RuntimeVal::Bool(_)) => todo!(),
        }
    }
    fn power(self, other: RuntimeVal) -> RuntimeVal {
        match (self, other) {
            (RuntimeVal::Number(lhs), RuntimeVal::Number(rhs)) => RuntimeVal::Number(lhs.powf(rhs)),

            (RuntimeVal::Number(lhs), RuntimeVal::Expression(rhs)) => {
                return AlgebraExpr::Exponent(Box::new(lhs.into()), Box::new(rhs)).into();
            }

            (RuntimeVal::Expression(rhs), RuntimeVal::Number(lhs)) => {
                return AlgebraExpr::Exponent(Box::new(rhs), Box::new(lhs.into())).into();
            }

            (RuntimeVal::Expression(mut lhs), RuntimeVal::Expression(mut rhs)) => {
                match (&mut lhs, &mut rhs) {
                    _ => {
                        return AlgebraExpr::Exponent(Box::new(lhs), Box::new(rhs)).into();
                    }
                }
            }

            (RuntimeVal::Expression(_), RuntimeVal::Bool(_)) => todo!(),
            (RuntimeVal::Unit, RuntimeVal::Unit) => todo!(),
            (RuntimeVal::Unit, RuntimeVal::Number(_)) => todo!(),
            (RuntimeVal::Unit, RuntimeVal::Expression(_)) => todo!(),
            (RuntimeVal::Unit, RuntimeVal::Bool(_)) => todo!(),
            (RuntimeVal::Number(_), RuntimeVal::Unit) => todo!(),
            (RuntimeVal::Expression(_), RuntimeVal::Unit) => todo!(),
            (RuntimeVal::Bool(_), RuntimeVal::Unit) => todo!(),
            (RuntimeVal::Bool(_), RuntimeVal::Number(_)) => todo!(),
            (RuntimeVal::Bool(_), RuntimeVal::Expression(_)) => todo!(),
            (RuntimeVal::Bool(_), RuntimeVal::Bool(_)) => todo!(),
            (RuntimeVal::Number(_), RuntimeVal::Bool(_)) => todo!(),
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

#[derive(Clone, Debug)]
pub enum AlgebraExpr {
    Sum(Vec<AlgebraExpr>),
    Product(Vec<AlgebraExpr>),
    Exponent(Box<AlgebraExpr>, Box<AlgebraExpr>),
    Number(f64),
    Symbol(String),
}

impl AlgebraExpr {
    fn simplify(&mut self) {
        /*let test = self.struct_equal(&mut AlgebraExpr::Sum(vec![
            AlgebraExpr::Product(vec![(3.0).into(), AlgebraExpr::Symbol("x".to_owned())]),
            AlgebraExpr::Product(vec![(2.0).into(), AlgebraExpr::Symbol("x".to_owned())]),
        ]));

        dbg!(test);*/

        self.add_like_terms();
        dbg!(&self);
        self.add_integers();
        dbg!(&self);
    }

    fn add_integers(&mut self) {
        dbg!(&self);
        match self {
            AlgebraExpr::Sum(terms) => {
                if terms.len() == 1 {
                    *self = terms[0].clone();
                    return;
                }

                let mut total = 0.0;

                let mut i = 0;
                while i < terms.len() {
                    if let AlgebraExpr::Number(n) = terms[i] {
                        total += n;
                        terms.remove(i);
                    } else {
                        i += 1;
                    }
                }

                let constant = AlgebraExpr::Number(total);
                if terms.is_empty() {
                    *self = constant
                } else if total != 0.0 {
                    terms.push(constant);
                }
            }
            AlgebraExpr::Product(factors) => {
                if factors.len() == 1 {
                    *self = factors[0].clone();
                    return;
                }

                let mut total = 1.0;

                let mut i = 0;
                while i < factors.len() {
                    if let AlgebraExpr::Number(n) = factors[i] {
                        total *= n;
                        factors.remove(i);
                    } else {
                        i += 1;
                    }
                }

                dbg!(&total);
                if total != 1.0 {
                    dbg!(&total);
                    factors.push(AlgebraExpr::Number(total));
                }
            }
            AlgebraExpr::Exponent(_, _) => todo!(),
            AlgebraExpr::Number(_) | AlgebraExpr::Symbol(_) => {}
        }
    }

    fn add_like_terms(&mut self) {
        match self {
            Self::Sum(terms) => {
                let mut term_coefficients = Vec::new();

                // Extract the coefficients from each term
                for term in terms {
                    let mut co_efficient = 1.0;
                    let mut index = 0;
                    if let AlgebraExpr::Product(factors) = term {
                        while index < factors.len() {
                            let factor = &factors[index];
                            if let AlgebraExpr::Number(n) = factor {
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
                let mut new_terms: Vec<AlgebraExpr> = Vec::new();

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
                        let mut term = AlgebraExpr::Product(vec![coefficient_total.into(), term]);
                        term.simplify();
                        new_terms.push(term);
                    }
                    dbg!(&new_terms);
                }
                *self = AlgebraExpr::Sum(new_terms);
            }
            _ => {}
        }
    }

    fn struct_equal(&mut self, other: &mut AlgebraExpr) -> bool {
        match (self, other) {
            (AlgebraExpr::Sum(terms), AlgebraExpr::Sum(other)) => {
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
            (AlgebraExpr::Product(factors), AlgebraExpr::Product(other)) => {
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
            (AlgebraExpr::Exponent(_, _), AlgebraExpr::Exponent(_, _)) => todo!(),
            (AlgebraExpr::Number(num), AlgebraExpr::Number(other)) => num == other,
            (AlgebraExpr::Symbol(symbol), AlgebraExpr::Symbol(other)) => symbol == other,
            _ => false,
        }
    }
}

impl Into<AlgebraExpr> for String {
    fn into(self) -> AlgebraExpr {
        AlgebraExpr::Symbol(self)
    }
}

impl Into<AlgebraExpr> for f64 {
    fn into(self) -> AlgebraExpr {
        AlgebraExpr::Number(self)
    }
}

impl Into<RuntimeVal> for AlgebraExpr {
    fn into(self) -> RuntimeVal {
        RuntimeVal::Expression(self)
    }
}
