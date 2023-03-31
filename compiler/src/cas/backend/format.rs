use matex_common::node::Precedence;

use crate::cas::backend::value::{Factors, Terms};

use super::value::RuntimeVal;

pub trait ValueFormatter {
    fn format(value: &RuntimeVal) -> String;
}

pub struct NormalFormatter;

impl NormalFormatter {
    fn format_impl(value: &RuntimeVal, prec: Precedence) -> String {
        use RuntimeVal::*;
        match value {
            Unit => "Unit value".to_owned(),
            Number(n) if prec == Precedence::Term => {
                if n.is_sign_negative() {
                    format!(" - {}", -n)
                } else {
                    format!(" + {}", n)
                }
            }
            Number(n) => n.to_string(),
            Symbol(s) => {
                if prec == Precedence::Term {
                    format!(" + {}", s)
                } else {
                    s.clone()
                }
            }
            Bool(b) => format!(" + {}", b),
            Sum(Terms(terms)) => {
                let mut buffer = String::new();
                for (i, term) in terms.iter().enumerate() {
                    if i == 0 {
                        buffer.push_str(&Self::format(term));
                    } else {
                        buffer.push_str(&Self::format_impl(term, Precedence::Term));
                    }
                }

                if prec > Precedence::Term {
                    return format!("({})", buffer);
                }

                buffer
            }
            Product(Factors(factors)) => {
                let mut vec = Vec::new();

                let mut negative_term = false;

                for factor in factors {
                    if let Number(n) = factor {
                        if n.is_sign_negative() {
                            negative_term = !negative_term;
                            if *n == -1.0 {
                                continue;
                            }
                            vec.push(Self::format_impl(&Number(-n), Precedence::Factor));
                            continue;
                        }
                    }
                    vec.push(Self::format_impl(factor, Precedence::Factor));
                }
                let mul_string = vec.join(" * ");

                if prec == Precedence::Term {
                    format!(" {} {}", if negative_term { "-" } else { "+" }, mul_string)
                } else {
                    format!("{}{}", if negative_term { "-" } else { "" }, mul_string)
                }
            }
            Exponent(base, exp) => {
                let base_str = Self::format(base);
                let exp_str = Self::format(exp);

                let str = base_str + "^" + exp_str.as_str();
                if prec == Precedence::Term {
                    format!(" + {}", str)
                } else {
                    str
                }
            }
        }
    }
}

impl ValueFormatter for NormalFormatter {
    fn format(value: &RuntimeVal) -> String {
        NormalFormatter::format_impl(value, Precedence::None)
    }
}
