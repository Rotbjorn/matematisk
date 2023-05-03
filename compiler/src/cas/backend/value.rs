use std::{cmp::Ordering, fmt::{Debug, self}};

#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};

use log::{debug};
use super::format::ValueFormatter;

macro_rules! value_debug {
    ($($arg:tt)+) => (debug!(target: "matex::value", $($arg)+));
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

    Sum(Terms),
    Product(Factors),
    Exponent(Box<RunVal>, Box<RunVal>),

    Function(String, Vec<RunVal>)
}


#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
pub struct Factors(pub Vec<RunVal>);

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
pub struct Terms(pub Vec<RunVal>);

impl RunVal {
    pub fn format<T: ValueFormatter>(&self) {
        T::format(&self);
    }

    pub(crate) fn new(typ: RunType) -> Self{
        Self {
            simplified: false,
            typ,
        }
    }

    pub(crate) fn add(mut self, mut other: RunVal) -> RunVal {
        value_debug!("add: {:?} + {:?}", self, other);
        use RunType::*;

        let typ = match (&mut self.typ, &mut other.typ) {
            (Unit, _) | (_, Unit)
            | (Undefined, _) | (_, Undefined) => Undefined.into(),

            (Bool(_), _) | (_, Bool(_)) => panic!("No addition with booleans"),

            (Number(lhs), Number(rhs)) => Number(*lhs + *rhs),

            (Sum(Terms(v)), _) => {
                v.push(other);
                self.typ
            }

            (_, Sum(Terms(v))) => {
                v.push(self);
                other.typ
            }

            (Number(_), Symbol(_))
            | (Number(_), Product(_))
            | (Number(_), Exponent(_, _))
            | (Number(_), Function(_, _))
            | (Symbol(_), Number(_))
            | (Symbol(_), Symbol(_))
            | (Symbol(_), Product(_))
            | (Symbol(_), Exponent(_, _))
            | (Symbol(_), Function(_, _)) 
            | (Product(_), Number(_))
            | (Product(_), Symbol(_))
            | (Product(_), Product(_))
            | (Product(_), Exponent(_, _))
            | (Product(_), Function(_, _)) 
            | (Exponent(_, _), Number(_))
            | (Exponent(_, _), Symbol(_))
            | (Exponent(_, _), Product(_))
            | (Exponent(_, _), Exponent(_, _))
            | (Exponent(_, _), Function(_, _)) 
            | (Function(_, _), Number(_)) 
            | (Function(_, _), Symbol(_)) 
            | (Function(_, _), Product(_))
            | (Function(_, _), Exponent(_, _))
            | (Function(_, _), Function(_, _)) => {
                RunType::Sum(Terms(Vec::from([self, other])))
            }
        };

         RunVal::new(typ)
    }
    pub(crate) fn multiply(mut self, mut other: RunVal) -> RunVal {
        value_debug!("multiply: {:?} * {:?}", self, other);
        use RunType::*;
        let typ = match (&mut self.typ, &mut other.typ) {
            (Unit, _) | (_, Unit) 
            | (Undefined, _) | (_, Undefined)=> Undefined,

            (Bool(_), _) | (_, Bool(_)) => panic!("No multiplication with booleans"),

            (Number(lhs), Number(rhs)) => Number(*lhs * *rhs),

            (Product(Factors(v)), _) => {
                v.push(other);
                self.typ
            }

            (_, Product(Factors(v))) => {
                v.push(self);
                other.typ
            }

            (Number(_), Symbol(_))
            | (Number(_), Sum(_))
            | (Number(_), Exponent(_, _))
            | (Number(_), Function(_, _))
            | (Symbol(_), Number(_))
            | (Symbol(_), Symbol(_))
            | (Symbol(_), Sum(_))
            | (Symbol(_), Exponent(_, _))
            | (Symbol(_), Function(_, _))
            | (Sum(_), Number(_))
            | (Sum(_), Symbol(_))
            | (Sum(_), Sum(_))
            | (Sum(_), Exponent(_, _))
            | (Sum(_), Function(_, _)) 
            | (Exponent(_, _), Number(_))
            | (Exponent(_, _), Symbol(_))
            | (Exponent(_, _), Sum(_))
            | (Exponent(_, _), Exponent(_, _))
            | (Exponent(_, _), Function(_, _))
            | (Function(_, _), Number(_))
            | (Function(_, _), Symbol(_))
            | (Function(_, _), Sum(_))
            | (Function(_, _), Exponent(_, _))
            | (Function(_, _), Function(_, _)) => {
                RunType::Product(Factors(Vec::from([self, other])))
            }
        };

        RunVal::new(typ)
    }
    pub(crate) fn power(self, other: RunVal) -> RunVal {
        value_debug!("power: {:?} ^ {:?}", self, other);
        use RunType::*;
        match (&self.typ, &other.typ) {
            (Unit, _) | (_, Unit)
            | (Undefined, _) | (_, Undefined) => Undefined.into(),

            (Bool(_), _) | (_, Bool(_)) => panic!("No powering with booleans"),

            // TODO: Calculate directly or keep as exponent?
            (Number(_lhs), Number(_rhs)) //=> Number(lhs.powf(*rhs)).into(),
                                                     => Exponent(Box::new(self), Box::new(other)).into(),
            (Number(_), Symbol(_))
            | (Number(_), Sum(_))
            | (Number(_), Product(_))
            | (Number(_), Exponent(_, _))
            | (Number(_), Function(_, _)) 
            | (Symbol(_), Number(_))
            | (Symbol(_), Symbol(_))
            | (Symbol(_), Sum(_))
            | (Symbol(_), Product(_))
            | (Symbol(_), Exponent(_, _))
            | (Symbol(_), Function(_, _)) => todo!(),
            | (Sum(_), Number(_))
            | (Sum(_), Symbol(_))
            | (Sum(_), Sum(_))
            | (Sum(_), Product(_))
            | (Sum(_), Exponent(_, _))
            | (Sum(_), Function(_, _)) 
            | (Product(_), Number(_))
            | (Product(_), Symbol(_))
            | (Product(_), Sum(_))
            | (Product(_), Product(_))
            | (Product(_), Exponent(_, _))
            | (Product(_), Function(_, _)) 
            | (Exponent(_, _), Number(_))
            | (Exponent(_, _), Symbol(_))
            | (Exponent(_, _), Sum(_))
            | (Exponent(_, _), Product(_))
            | (Exponent(_, _), Exponent(_, _))
            | (Exponent(_, _), Function(_, _)) 
            | (Function(_, _), Number(_)) 
            | (Function(_, _), Symbol(_)) 
            | (Function(_, _), Sum(_)) 
            | (Function(_, _), Product(_)) 
            | (Function(_, _), Exponent(_, _)) 
            | (Function(_, _), Function(_, _)) => Exponent(Box::new(self), Box::new(other)).into()
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


                let Factors(factors) = factors;

                if coeff != 1.0 {
                    factors.push(Number(coeff).into());
                }
            }
            Exponent(base, exp) => {
                base.simplify();
                exp.simplify();
            }
            _ => {}
        }
        self.flatten();
        self.simplified = true;
        dbg!(&self);
    }

    pub(crate) fn combine_like_terms(terms: &mut Terms) {
        value_debug!("combining like terms");
        use RunType::*;
        // Extract the coefficients from each term
        if terms.0.len() <= 1 {
            value_debug!("returning; only one item, nothing to combine.");
            return;
        }
        
        let mut term_coefficients = RunVal::extract_coefficients(terms);

        value_debug!("term_coefficients: {:?}", term_coefficients);
        value_debug!("terms: {:?}", terms);

        let mut new_terms: Vec<RunVal> = Vec::new();

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
                new_terms.push(term);
            } else if coefficient_total != 0.0 {
                let mut term: RunVal =
                    RunType::Product(Factors(Vec::from([Number(coefficient_total).into(), term]))).into();
                    
                term.simplify();                

                if coefficient_total.is_sign_positive() {
                    new_terms.push(term);
                } else {
                    new_terms.push(term);
                }
            }
            dbg!(&new_terms);
        }
        *terms = Terms(new_terms);
    }

    pub(crate) fn combine_like_factors(factors: &mut Factors) {
        value_debug!("combining like factors");
        let factors_vec = &mut factors.0;
        let mut new_factors: Vec<RunVal> = Vec::new();

        while let Some(factor) = factors_vec.pop() {

            let mut exponents: Vec<RunVal> = Vec::new();

            let mut found = false;

            let base = if let RunType::Exponent(base, exp) = factor.typ {
                exponents.push(*exp);
                *base
            } else {
                exponents.push(RunType::Number(1.0).into());
                factor
            };

            let mut i = 0;
            while i < factors_vec.len() {
                let other_factor = &factors_vec[i];                

                if base.struct_equal(other_factor) {
                    found = true;
                    exponents.push(RunType::Number(1.0).into());
                    factors_vec.remove(i);
                    continue
                } else if let RunType::Exponent(other_base, other_exp) = &other_factor.typ {
                    if base.struct_equal(other_base) {
                        found = true;
                        exponents.push(*other_exp.clone());
                        factors_vec.remove(i);
                        continue
                    }
                }
                i += 1;
            }

            if found {
                let mut exponents = RunVal::new(RunType::Sum(Terms(exponents)));
                value_debug!("exponents before simplification: {:?}", &exponents);
                exponents.simplify();
                value_debug!("exponents after simplification: {:?}", &exponents);
                let exponent = RunType::Exponent(Box::new(base), Box::new(exponents));
                new_factors.push(exponent.into());
            } else {
                new_factors.push(base.into()); 
            }
        }
        *factors_vec = new_factors;
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

        if total == 0.0 && terms.len() != 0 {
            return;
            // If there are no terms left, then add the zero as the only term.
        }

        let constant = RunType::Number(total);

        terms.push(constant.into());
    }

    #[allow(dead_code)]
    pub(crate) fn rearrange(Terms(terms): &mut Terms) {
        use RunType::*;
        terms.sort_by(|a, b| {
            dbg!(&a);
            dbg!(&b);
            match (&a.typ, &b.typ) {
                (_, Product(factors)) => {
                    return if RunVal::product_is_negative(factors) {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }
                (Product(factors), _) if RunVal::product_is_negative(factors) => {
                    Ordering::Greater
                }
                (_, _) => Ordering::Less,
            }
        });
        dbg!(&terms);
    }

    pub(crate) fn flatten(&mut self) {
        use RunType::*;
        match &mut self.typ {
            Sum(Terms(v)) 
            | Product(Factors(v)) 
            if v.len() == 1 => {
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

                // Verify that the vectors `factors` and `other` are structurally equal:
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
                base.struct_equal(&other_base) && exp.struct_equal(&other_exp)
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

            dbg!(&factor);

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

    #[allow(dead_code)]
    pub(crate) fn product_is_negative(Factors(factors): &Factors) -> bool {
        value_debug!("is product negative?");
        use RunType::*;

        let is_negative = factors.iter().fold(false, |is_negative, factor| {
            if let Number(n) = factor.typ {
                if n.is_sign_negative() {
                    return !is_negative;
                }
            }
            is_negative
        });

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
            RunType::Sum(Terms(terms)) => {
                write!(f, "(+, ")?;
                let mut vec = Vec::<String>::new();

                for term in terms {
                    vec.push(format!("{:?}", term));
                }

                let str = vec.join(", ");
                f.write_str(&str)?;

                write!(f, ")")
            },

            RunType::Product(Factors(factors)) => {
                write!(f, "(*, ")?;
                let mut vec = Vec::<String>::new();

                for factor in factors {
                    vec.push(format!("{:?}", factor));
                }

                let str = vec.join(", ");
                f.write_str(&str)?;

                write!(f, ")")
            },

            RunType::Exponent(base, exp) => {
                write!(f, "(^, ({:?}), ({:?}))", base, exp)
            },

            RunType::Function(name, args) => {
                write!(f, "{}({:?})", name, args)
            },
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
