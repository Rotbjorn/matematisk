use matex_common::node::{Expr, OperationType};

pub fn combine_like_terms(expression: &mut Expr) -> (f64, Expr) {
    let (coefficient, product) = match expression {
        Expr::BinaryOp {
            left,
            operation,
            right,
        } => {
            let (lhs_coefficient, lhs_product) = combine_like_terms(left);
            let (rhs_coefficient, rhs_product) = combine_like_terms(right);
            (
                lhs_coefficient * rhs_coefficient,
                Expr::BinaryOp {
                    left: Box::new(lhs_product),
                    operation: OperationType::Multiply,
                    right: Box::new(rhs_product),
                },
            )
        }
        Expr::Number(n) => (*n, expression.clone()),
        _ => (1.0, expression.clone()),
    };
    println!("Test: {} -> {:?}", coefficient, product);
    //    if let Expr::Variable(var) = product {
    //        (coefficient, Expr::BinaryOp { left: Box::new(Expr::Number(coefficient.into())), operation: OperationType::Multiply, right: Box::new(Expr::Variable(var)) } )
    //    } else {
    //        (coefficient, product)
    //    }
    if coefficient == 1.0 {
        (coefficient, product)
    } else {
        (
            coefficient,
            Expr::BinaryOp {
                left: Box::new(Expr::Number(coefficient)),
                operation: OperationType::Multiply,
                right: Box::new(product),
            },
        )
    }
}
