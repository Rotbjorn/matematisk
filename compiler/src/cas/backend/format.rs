use matex_common::node::Precedence;

use crate::cas::backend::value::{Factors, Terms};

use super::value::{RunType, RunVal};

pub trait ValueFormatter {
    fn format(value: &RunVal) -> String;
}

pub struct NormalFormatter;

impl NormalFormatter {
    fn format_impl(value: &RunVal, prec: Precedence) -> String {
        use RunType::*;
        match &value.typ {
            Unit => "Unit value".to_owned(),
            Undefined => "Undefined?".to_owned(),
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
                    if let Number(n) = factor.typ {
                        if n.is_sign_negative() {
                            negative_term = !negative_term;
                            if n == -1.0 {
                                continue;
                            }
                            vec.push(Self::format_impl(&Number(-n).into(), Precedence::Factor));
                            continue;
                        }
                    }
                    vec.push(Self::format_impl(factor, Precedence::Factor));
                }
                let mul_string = vec.join(" * ");

                if prec == Precedence::Term {
                    format!(" {} {}", if negative_term { "-" } else { "+" }, mul_string)
                } else if prec == Precedence::Exponent {
                    format!("({})", mul_string)
                } else {
                    format!("{}{}", if negative_term { "-" } else { "" }, mul_string)
                }
            }
            Exponent(base, exp) => {
                let base_str = Self::format_impl(base, Precedence::Exponent);
                let exp_str = Self::format_impl(exp, Precedence::Exponent);

                let str = base_str + "^" + exp_str.as_str();
                if prec == Precedence::Term {
                    format!(" + {}", str)
                } else if prec == Precedence::Exponent {
                    format!("({})", str)
                } else {
                    str
                }
            }
            Function(name, arguments) => {
                let mut args = Vec::new();

                for argument in arguments {
                    let argument_str = Self::format(argument);
                    args.push(argument_str); 
                }

                format!("{}({})", name, args.join(", "))
            }
        }
    }
}

impl ValueFormatter for NormalFormatter {
    fn format(value: &RunVal) -> String {
        NormalFormatter::format_impl(value, Precedence::None)
    }
}
