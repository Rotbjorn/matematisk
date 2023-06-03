use std::{
    cmp::Ordering,
    fmt::{self, Debug},
};

#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};

use super::format::ValueFormatter;
use log::{debug, error};

macro_rules! value_debug {
    ($($arg:tt)+) => (debug!(target: "matex::value", "[{}:{}] {}", file!(), line!(), &format!($($arg)+)));
}

#[allow(unused_macros)]
macro_rules! value_error {
    ($($arg:tt)+) => (error!(target: "matex::value", $($arg)+));
}

#[derive(Clone, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
// Better name
pub struct RunVal {
    pub(crate) simplified: bool,
    pub typ: RunType,
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq)]
pub enum RunType {
    // Merge these two?
    Unit,

    Undefined,

    // TODO: Add complex, real, etc
    Number(f64),
    Symbol(String),
    Bool(bool),
    Vector(Vec<RunVal>),

    Sum(Terms),
    Product(Factors),
    Exponent(Box<RunVal>, Box<RunVal>),

    Function(String, Vec<RunVal>),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
pub struct Factors(pub Vec<RunVal>);

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
pub struct Terms(pub Vec<RunVal>);

impl RunVal {
    pub fn format<T: ValueFormatter>(&self) {
        T::format(self);
    }

    pub(crate) fn new(typ: RunType) -> Self {
        Self {
            simplified: false,
            typ,
        }
    }

    // TODO: Make these take references and return new one...
    pub(crate) fn add(self, other: RunVal) -> RunVal {
        value_debug!("add: {:?} + {:?}", self, other);
        use RunType::*;

        let typ = match (self.typ, other.typ) {
            (Unit, _) | (_, Unit) | (Undefined, _) | (_, Undefined) => Undefined,

            (Bool(_), _) | (_, Bool(_)) => panic!("No addition with booleans"),

            (Number(lhs), Number(rhs)) => Number(lhs + rhs),

            (Sum(Terms(mut v)), Sum(Terms(other_v))) => {
                v.extend(other_v);
                Sum(Terms(v))
            }

            (Sum(Terms(mut v)), o) => {
                v.push(o.into());
                Sum(Terms(v))
            }

            (s, Sum(Terms(mut v))) => {
                v.push(s.into());
                Sum(Terms(v))
            }

            (s @ Number(_), o @ Symbol(_))
            | (s @ Number(_), o @ Product(_))
            | (s @ Number(_), o @ Exponent(_, _))
            | (s @ Number(_), o @ Function(_, _))
            | (s @ Symbol(_), o @ Number(_))
            | (s @ Symbol(_), o @ Symbol(_))
            | (s @ Symbol(_), o @ Product(_))
            | (s @ Symbol(_), o @ Exponent(_, _))
            | (s @ Symbol(_), o @ Function(_, _))
            | (s @ Product(_), o @ Number(_))
            | (s @ Product(_), o @ Symbol(_))
            | (s @ Product(_), o @ Product(_))
            | (s @ Product(_), o @ Exponent(_, _))
            | (s @ Product(_), o @ Function(_, _))
            | (s @ Exponent(_, _), o @ Number(_))
            | (s @ Exponent(_, _), o @ Symbol(_))
            | (s @ Exponent(_, _), o @ Product(_))
            | (s @ Exponent(_, _), o @ Exponent(_, _))
            | (s @ Exponent(_, _), o @ Function(_, _))
            | (s @ Function(_, _), o @ Number(_))
            | (s @ Function(_, _), o @ Symbol(_))
            | (s @ Function(_, _), o @ Product(_))
            | (s @ Function(_, _), o @ Exponent(_, _))
            | (s @ Function(_, _), o @ Function(_, _)) => {
                RunType::Sum(Terms(Vec::from([s.into(), o.into()])))
            }
            (Number(_), Vector(_)) => todo!(),
            (Symbol(_), Vector(_)) => todo!(),
            (Vector(_), Number(_)) => todo!(),
            (Vector(_), Symbol(_)) => todo!(),
            (Vector(_), Vector(_)) => todo!(),
            (Vector(_), Product(_)) => todo!(),
            (Vector(_), Exponent(_, _)) => todo!(),
            (Vector(_), Function(_, _)) => todo!(),
            (Product(_), Vector(_)) => todo!(),
            (Exponent(_, _), Vector(_)) => todo!(),
            (Function(_, _), Vector(_)) => todo!(),
        };

        RunVal::new(typ)
    }

    pub(crate) fn multiply(self, other: RunVal) -> RunVal {
        value_debug!("multiply: {:?} * {:?}", self, other);
        use RunType::*;
        let typ = match (self.typ, other.typ) {
            (Unit, _) | (_, Unit) | (Undefined, _) | (_, Undefined) => Undefined,

            (Bool(_), _) | (_, Bool(_)) => panic!("No multiplication with booleans"),

            (Number(lhs), Number(rhs)) => Number(lhs * rhs),

            (Product(Factors(mut v)), Product(Factors(other_v))) => {
                v.extend(other_v);
                Product(Factors(v))
            }

            (Product(Factors(mut v)), o) => {
                v.push(o.into());
                Product(Factors(v))
            }

            (o, Product(Factors(mut v))) => {
                v.push(o.into());
                Product(Factors(v))
            }

            (s @ Number(_), o @ Symbol(_))
            | (s @ Number(_), o @ Sum(_))
            | (s @ Number(_), o @ Exponent(_, _))
            | (s @ Number(_), o @ Function(_, _))
            | (s @ Symbol(_), o @ Number(_))
            | (s @ Symbol(_), o @ Symbol(_))
            | (s @ Symbol(_), o @ Sum(_))
            | (s @ Symbol(_), o @ Exponent(_, _))
            | (s @ Symbol(_), o @ Function(_, _))
            | (s @ Sum(_), o @ Number(_))
            | (s @ Sum(_), o @ Symbol(_))
            | (s @ Sum(_), o @ Sum(_))
            | (s @ Sum(_), o @ Exponent(_, _))
            | (s @ Sum(_), o @ Function(_, _))
            | (s @ Exponent(_, _), o @ Number(_))
            | (s @ Exponent(_, _), o @ Symbol(_))
            | (s @ Exponent(_, _), o @ Sum(_))
            | (s @ Exponent(_, _), o @ Exponent(_, _))
            | (s @ Exponent(_, _), o @ Function(_, _))
            | (s @ Function(_, _), o @ Number(_))
            | (s @ Function(_, _), o @ Symbol(_))
            | (s @ Function(_, _), o @ Sum(_))
            | (s @ Function(_, _), o @ Exponent(_, _))
            | (s @ Function(_, _), o @ Function(_, _)) => {
                RunType::Product(Factors(Vec::from([s.into(), o.into()])))
            }

            (Number(_), Vector(_)) => todo!(),
            (Symbol(_), Vector(_)) => todo!(),
            (Vector(_), Number(_)) => todo!(),
            (Vector(_), Symbol(_)) => todo!(),
            (Vector(_), Vector(_)) => todo!(),
            (Vector(_), Sum(_)) => todo!(),
            (Vector(_), Exponent(_, _)) => todo!(),
            (Vector(_), Function(_, _)) => todo!(),
            (Sum(_), Vector(_)) => todo!(),
            (Exponent(_, _), Vector(_)) => todo!(),
            (Function(_, _), Vector(_)) => todo!(),
        };

        RunVal::new(typ)
    }
    pub(crate) fn power(self, other: RunVal) -> RunVal {
        value_debug!("power: {:?} ^ {:?}", self, other);
        use RunType::*;
        match (self.typ, other.typ) {
            (Unit, _) | (_, Unit) | (Undefined, _) | (_, Undefined) => Undefined.into(),

            (Bool(_), _) | (_, Bool(_)) => panic!("No powering with booleans"),

            // TODO: Calculate directly or keep as exponent?
            (Number(lhs), Number(rhs)) => Number(lhs.powf(rhs)).into(),
            // (s@Number(_), o@Number(_)) => Exponent(Box::new(s.into()), Box::new(o.into())).into(),
            (Exponent(base, exp), o @ Number(_))
            | (Exponent(base, exp), o @ Symbol(_))
            | (Exponent(base, exp), o @ Sum(_))
            | (Exponent(base, exp), o @ Product(_))
            | (Exponent(base, exp), o @ Exponent(_, _))
            | (Exponent(base, exp), o @ Function(_, _)) => {
                Exponent(base, Box::new(exp.multiply(o.into()))).into()
            }

            (s @ Number(_), o @ Symbol(_))
            | (s @ Number(_), o @ Sum(_))
            | (s @ Number(_), o @ Product(_))
            | (s @ Number(_), o @ Exponent(_, _))
            | (s @ Number(_), o @ Function(_, _))
            | (s @ Symbol(_), o @ Number(_))
            | (s @ Symbol(_), o @ Symbol(_))
            | (s @ Symbol(_), o @ Sum(_))
            | (s @ Symbol(_), o @ Product(_))
            | (s @ Symbol(_), o @ Exponent(_, _))
            | (s @ Symbol(_), o @ Function(_, _))
            | (s @ Sum(_), o @ Number(_))
            | (s @ Sum(_), o @ Symbol(_))
            | (s @ Sum(_), o @ Sum(_))
            | (s @ Sum(_), o @ Product(_))
            | (s @ Sum(_), o @ Exponent(_, _))
            | (s @ Sum(_), o @ Function(_, _))
            | (s @ Product(_), o @ Number(_))
            | (s @ Product(_), o @ Symbol(_))
            | (s @ Product(_), o @ Sum(_))
            | (s @ Product(_), o @ Product(_))
            | (s @ Product(_), o @ Exponent(_, _))
            | (s @ Product(_), o @ Function(_, _))
            | (s @ Function(_, _), o @ Number(_))
            | (s @ Function(_, _), o @ Symbol(_))
            | (s @ Function(_, _), o @ Sum(_))
            | (s @ Function(_, _), o @ Product(_))
            | (s @ Function(_, _), o @ Exponent(_, _))
            | (s @ Function(_, _), o @ Function(_, _)) => {
                Exponent(Box::new(s.into()), Box::new(o.into())).into()
            }

            (Number(_), Vector(_)) => todo!(),
            (Symbol(_), Vector(_)) => todo!(),
            (Vector(_), Number(_)) => todo!(),
            (Vector(_), Symbol(_)) => todo!(),
            (Vector(_), Vector(_)) => todo!(),
            (Vector(_), Sum(_)) => todo!(),
            (Vector(_), Product(_)) => todo!(),
            (Vector(_), Exponent(_, _)) => todo!(),
            (Vector(_), Function(_, _)) => todo!(),
            (Sum(_), Vector(_)) => todo!(),
            (Product(_), Vector(_)) => todo!(),
            (Exponent(_, _), Vector(_)) => todo!(),
            (Function(_, _), Vector(_)) => todo!(),
        }
    }
    pub(crate) fn less(self, other: RunVal) -> RunVal {
        match (&self.typ, &other.typ) {
            (RunType::Number(lhs), RunType::Number(rhs)) => RunType::Bool(lhs < rhs).into(),

            _ => {
                todo!("Not implemented: {:?} {:?}", self, other);
            }
        }
    }
    pub(crate) fn equal(self, other: RunVal) -> RunVal {
        match (&self.typ, &other.typ) {
            (RunType::Number(lhs), RunType::Number(rhs)) => RunType::Bool(lhs == rhs).into(),

            _ => {
                todo!("Not implemented: {:?} {:?}", self, other);
            }
        }
    }
    pub(crate) fn less_equal(self, other: RunVal) -> RunVal {
        match (&self.typ, &other.typ) {
            (RunType::Number(lhs), RunType::Number(rhs)) => RunType::Bool(lhs <= rhs).into(),

            _ => {
                todo!("Not implemented: {:?} {:?}", self, other);
            }
        }
    }
    pub(crate) fn greater(self, other: RunVal) -> RunVal {
        match (&self.typ, &other.typ) {
            (RunType::Number(lhs), RunType::Number(rhs)) => RunType::Bool(lhs > rhs).into(),

            _ => {
                todo!("Not implemented: {:?} {:?}", self, other);
            }
        }
    }
    pub(crate) fn greater_equal(self, other: RunVal) -> RunVal {
        match (&self.typ, &other.typ) {
            (RunType::Number(lhs), RunType::Number(rhs)) => RunType::Bool(lhs >= rhs).into(),

            _ => {
                todo!("Not implemented: {:?} {:?}", self, other);
            }
        }
    }
}

impl RunVal {
    pub(crate) fn simplify(&mut self) {
        value_debug!("simplify: {:?}", self);
        use RunType::*;

        if self.simplified {
            self.flatten();
            value_debug!("skipping simplification; already simplified!");
            return;
        }

        match &mut self.typ {
            Sum(terms) => {
                value_debug!("simplify sum");
                value_debug!("recursive simplify factors");
                for term in &mut terms.0 {
                    term.simplify();
                }

                //RunVal::combine_integers(terms);
                RunVal::combine_like_terms(terms);
                RunVal::combine_integers(terms);

                //RunVal::rearrange(terms);
            }
            Product(factors) => {
                value_debug!("simplify product");

                value_debug!("recursive simplify factors");
                for factor in &mut factors.0 {
                    factor.simplify();
                }

                let coeff = RunVal::extract_coefficient(factors);

                RunVal::combine_like_factors(factors);

                value_debug!("after combined: {:?}", factors);

                let Factors(factors) = factors;

                if coeff != 1.0 {
                    factors.push(Number(coeff).into());
                }
            }
            Exponent(base, exp) => {
                value_debug!("simplify exponent");
                base.simplify();
                exp.simplify();

                // Merge exponents, e.g. (e^a)^b => e^(ab)
                // TODO: Move to its own function?
                if let Exponent(b_base, b_exp) = base.typ.clone() {
                    let mut exponents: Box<RunVal> =
                        Box::new(RunType::Product(Factors(Vec::new())).into());
                    exponents = Box::new(exponents.multiply(*b_exp));
                    exponents = Box::new(exponents.multiply(*exp.clone()));
                    *base = b_base;
                    *exp = exponents;
                }
            }
            _ => {}
        }
        self.flatten();
        self.simplified = true;
        value_debug!("current self after simplify: {:?}", self);
    }

    pub(crate) fn combine_like_terms(terms: &mut Terms) {
        value_debug!("combining like terms");
        use RunType::*;
        if terms.0.len() <= 1 {
            value_debug!("returning; only one item, nothing to combine.");
            return;
        }

        // Extract the coefficients from each term
        let mut term_coefficients = RunVal::extract_coefficients(terms);

        value_debug!("term_coefficients: {:?}", term_coefficients);
        value_debug!("terms: {:?}", terms);

        let mut new_terms: RunVal = Sum(Terms(Vec::new())).into();

        while let Some((co_eff, term)) = term_coefficients.pop() {
            let mut coefficient_total = co_eff;

            let mut i = 0;
            while i < term_coefficients.len() {
                let (co_eff, other_term) = &term_coefficients[i];

                if term.struct_equal(other_term) {
                    coefficient_total += *co_eff;
                    term_coefficients.remove(i);
                } else {
                    i += 1;
                }
            }

            if coefficient_total == 1.0 {
                new_terms = new_terms.add(term);
            } else if coefficient_total != 0.0 {
                let mut term: RunVal =
                    RunType::Product(Factors(Vec::from([Number(coefficient_total).into(), term])))
                        .into();

                term.simplify();

                if coefficient_total.is_sign_positive() {
                    new_terms = new_terms.add(term);
                }
            }
            value_debug!("current new_terms: {:?}", new_terms);
        }
        let Sum(new_terms) = new_terms.typ else {
            value_error!("There is no way that we should end up here...");
            panic!("STOPPING");
        };
        *terms = new_terms
    }

    pub(crate) fn combine_like_factors(factors: &mut Factors) {
        value_debug!("combining like factors");
        use RunType::*;

        if factors.0.len() <= 1 {
            value_debug!("returning; only one item, nothing to combine.");
            return;
        }

        let factors_vec = &mut factors.0;
        let mut new_factors: Vec<RunVal> = Vec::new();

        while let Some(factor) = factors_vec.pop() {
            value_debug!("current factor: {:?}", factor);

            let mut exponents_vec: Vec<RunVal> = Vec::new();

            let mut found = false;

            let base = if let Exponent(base, exp) = factor.typ {
                value_debug!("starting with exp: {:?}", exp);
                exponents_vec.push(*exp);
                found = true;
                *base
            } else {
                exponents_vec.push(Number(1.0).into());
                factor
            };

            let mut exponents: RunVal = Sum(Terms(exponents_vec)).into();

            let mut i = 0;
            while i < factors_vec.len() {
                value_debug!("exponents: {:?}", exponents);
                let other_factor = &factors_vec[i];

                if base.struct_equal(other_factor) {
                    found = true;
                    exponents = exponents.add(Number(1.0).into());
                    factors_vec.remove(i);
                    continue;
                } else if let Exponent(other_base, other_exp) = &other_factor.typ {
                    if base.struct_equal(other_base) {
                        found = true;
                        // TODO: Take ownership instead?
                        exponents = exponents.add(*other_exp.clone());
                        factors_vec.remove(i);
                        continue;
                    }
                }
                i += 1;
            }

            if found {
                value_debug!("exponents before simplification: {:?}", &exponents);
                exponents.simplify();
                value_debug!("exponents after simplification: {:?}", &exponents);

                if let Number(n) = exponents.typ {
                    if n != 0.0 {
                        let exponent = Exponent(Box::new(base), Box::new(exponents));
                        new_factors.push(exponent.into());
                    } else {
                        new_factors.push(Number(1.0).into());
                    }
                } else {
                    let exponent = Exponent(Box::new(base), Box::new(exponents));
                    new_factors.push(exponent.into());
                }
            } else {
                new_factors.push(base);
            }
        }
        *factors_vec = new_factors;
        value_debug!("after combining like factors: {:?}", factors_vec);
    }

    pub(crate) fn combine_integers(Terms(terms): &mut Terms) {
        let mut total = 0.0;

        let mut i = 0;
        while i < terms.len() {
            if let RunType::Number(n) = terms[i].typ {
                total += n;
                terms.remove(i);
            } else {
                i += 1;
            }
        }

        if total == 0.0 && terms.is_empty() {
            // TODO: If there are no terms left, then add the zero as the only term?
            // Alternatively just check at printing if the most outer expression is an empty sum?
            return;
        }

        let constant = RunType::Number(total);

        terms.push(constant.into());
    }

    pub fn rearrange(&mut self) {
        use RunType::*;
        match &mut self.typ {
            Sum(Terms(terms)) => {
                terms.sort_by(|a, b| {
                    dbg!(&a);
                    dbg!(&b);
                    match (&a.typ, &b.typ) {
                        (_, o @ Number(_)) | (_, o @ Product(_)) => {
                            if RunVal::value_is_negative(o) {
                                Ordering::Less
                            } else {
                                Ordering::Greater
                            }
                        }
                        (s @ Number(_), _) | (s @ Product(_), _)
                            if RunVal::value_is_negative(s) =>
                        {
                            Ordering::Greater
                        }
                        (_, _) => Ordering::Less,
                    }
                });
            }
            Unit
            | Undefined
            | Number(_)
            | Symbol(_)
            | Bool(_)
            | Vector(_)
            | Product(_)
            | Exponent(_, _)
            | Function(_, _) => {}
        }
    }

    pub(crate) fn flatten(&mut self) {
        use RunType::*;
        match &mut self.typ {
            Sum(Terms(v)) | Product(Factors(v)) if v.len() == 1 => {
                let value = &mut v[0];
                value.flatten();
                *self = value.clone();
            }
            _ => {}
        }
    }

    pub(crate) fn struct_equal(&self, other: &RunVal) -> bool {
        match (&self.typ, &other.typ) {
            (RunType::Sum(Terms(terms)), RunType::Sum(Terms(other))) => {
                if terms.len() != other.len() {
                    return false;
                }

                let mut terms_remaining = other.iter().collect::<Vec<_>>();

                'outer: for term in terms {
                    for i in 0..terms_remaining.len() {
                        let other_term = &terms_remaining[i];
                        if term.struct_equal(other_term) {
                            terms_remaining.remove(i);
                            continue 'outer;
                        }
                    }

                    return false;
                }

                true
            }
            (RunType::Product(Factors(factors)), RunType::Product(Factors(other))) => {
                if factors.len() != other.len() {
                    return false;
                }

                let mut factors_remaining = other.iter().collect::<Vec<_>>();

                'outer: for factor in factors {
                    for i in 0..factors_remaining.len() {
                        let other_term = &factors_remaining[i];
                        if factor.struct_equal(other_term) {
                            factors_remaining.remove(i);
                            continue 'outer;
                        }
                    }

                    return false;
                }

                true
            }
            (RunType::Exponent(base, exp), RunType::Exponent(other_base, other_exp)) => {
                base.struct_equal(other_base) && exp.struct_equal(other_exp)
            }
            (RunType::Number(num), RunType::Number(other)) => num == other,
            (RunType::Symbol(symbol), RunType::Symbol(other)) => symbol == other,
            _ => false,
        }
    }

    pub(crate) fn extract_coefficients(Terms(terms): &mut Terms) -> Vec<(f64, RunVal)> {
        value_debug!("extracting coeffients: {:?}", terms);
        let mut term_coefficients: Vec<(f64, RunVal)> = Vec::new();

        for term in terms {
            value_debug!("current term: {:?}", term);

            let coeff = if let RunType::Product(ref mut factors) = term.typ {
                RunVal::extract_coefficient(factors)
            } else {
                1.0
            };

            term.simplify();
            term_coefficients.push((coeff, term.clone()));
            value_debug!("current term_coefficients: {:?}", term_coefficients);
        }
        term_coefficients
    }

    pub(crate) fn extract_coefficient(Factors(factors): &mut Factors) -> f64 {
        value_debug!("extracting coefficient of: {:?}", factors);
        let mut index = 0;
        let mut coeff = 1.0;

        while index < factors.len() {
            let factor = &factors[index];

            //value_debug!("current factor: {:?}", factor);

            if let RunType::Number(n) = &factor.typ {
                coeff *= *n;
                value_debug!("coeff: {}", coeff);
                value_debug!("current factor: {:?}", factor);
                factors.remove(index);
            } else {
                index += 1;
            }
        }
        value_debug!("extracted coefficient: {}", coeff);
        coeff
    }

    pub(crate) fn value_is_negative(typ: &RunType) -> bool {
        use RunType::*;
        value_debug!("is value negative?");
        let is_negative = match typ {
            Number(n) => n.is_sign_negative(),
            Product(Factors(factors)) => {
                let is_negative = factors.iter().fold(false, |is_negative, factor| {
                    if let Number(n) = factor.typ {
                        if n.is_sign_negative() {
                            return !is_negative;
                        }
                    }
                    is_negative
                });
                is_negative
            }
            Exponent(base, _) => RunVal::value_is_negative(&base.typ),

            Unit | Undefined | Vector(_) | Sum(_) | Function(_, _) | Symbol(_) | Bool(_) => false,
        };

        value_debug!("is negative: {}", is_negative);

        is_negative
    }
}

impl fmt::Debug for RunVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.typ {
            RunType::Unit
            | RunType::Undefined
            | RunType::Number(_)
            | RunType::Symbol(_)
            | RunType::Bool(_)
            | RunType::Function(_, _) => write!(f, "{:?}", self.typ),
            _ => {
                if self.simplified {
                    write!(f, "({:?})", self.typ)
                } else {
                    write!(f, "{{{:?}}}", self.typ)
                }
            }
        }
    }
}

impl From<RunType> for RunVal {
    fn from(value: RunType) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for RunType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunType::Unit => write!(f, "unit"),
            RunType::Undefined => write!(f, "undefined"),
            RunType::Number(n) => write!(f, "{}", n),
            RunType::Symbol(s) => write!(f, "'{}'", s),
            RunType::Bool(b) => write!(f, "{}", b),
            RunType::Vector(vec) => write!(f, "{:?}", vec),
            RunType::Sum(Terms(terms)) => {
                write!(f, "(+, ")?;
                let mut vec = Vec::<String>::new();

                for term in terms {
                    vec.push(format!("{:?}", term));
                }

                let str = vec.join(", ");
                f.write_str(&str)?;

                write!(f, ")")
            }

            RunType::Product(Factors(factors)) => {
                write!(f, "(*, ")?;
                let mut vec = Vec::<String>::new();

                for factor in factors {
                    vec.push(format!("{:?}", factor));
                }

                let str = vec.join(", ");
                f.write_str(&str)?;

                write!(f, ")")
            }

            RunType::Exponent(base, exp) => {
                write!(f, "(^, ({:?}), ({:?}))", base, exp)
            }

            RunType::Function(name, args) => {
                write!(f, "{}({:?})", name, args)
            }
        }
    }
}

impl From<String> for RunType {
    fn from(val: String) -> Self {
        RunType::Symbol(val)
    }
}

impl From<f64> for RunType {
    fn from(value: f64) -> Self {
        RunType::Number(value)
    }
}
