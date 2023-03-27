use matex_common::node::Precedence;

use super::runtime::RuntimeVal;

pub trait ValueFormatter {
    fn format(value: &RuntimeVal) -> String;
}

pub struct NormalFormatter;

impl NormalFormatter {
    fn format_impl(value: &RuntimeVal, prec: Precedence) -> String {
        use RuntimeVal::*;
        match value {
            Unit => "Unit value".to_owned(),
            Number(n) => n.to_string(),
            Symbol(s) => s.to_string(),
            Bool(b) => b.to_string(),
            Sum(terms) => {
                let mut vec = Vec::new();
                for term in terms {
                    vec.push(Self::format(term));
                }

                let string = vec.join("+");

                if prec > Precedence::Term {
                    return format!("({})", string); 
                }

                return vec.join("+");
            }
            Product(factors) => {
                if factors.len() == 2 {

                }
                let mut vec = Vec::new();
                for factor in factors {
                    vec.push(Self::format_impl(factor, Precedence::Factor));
                }
                return vec.join("*");
            }
            Exponent(base, exp) => {
                let base_str = Self::format(base);
                let exp_str = Self::format(exp);
                return base_str + "^" + exp_str.as_str();
            }
        }
    }
}

impl ValueFormatter for NormalFormatter {
    fn format(value: &RuntimeVal) -> String {
        NormalFormatter::format_impl(value, Precedence::None)
    }
}
