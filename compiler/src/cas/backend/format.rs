use super::runtime::RuntimeVal;

pub trait ValueFormatter {
    fn format(value: &RuntimeVal) -> String;
}

pub struct NormalFormatter;

impl ValueFormatter for NormalFormatter {
    fn format(value: &RuntimeVal) -> String {
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
                return vec.join("+");
            }
            Product(factors) => {
                let mut vec = Vec::new();
                for factor in factors {
                    vec.push(Self::format(factor));
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
