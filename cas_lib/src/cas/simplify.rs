use super::node::{Node, OperationType};

pub fn combine_like_terms(expression: &mut Node) -> (f64, Node) {
    let (coefficient, product) = match expression {
        Node::BinaryOp {
            left,
            operation,
            right,
        } => {
            let (lhs_coefficient, lhs_product) = combine_like_terms(left);
            let (rhs_coefficient, rhs_product) = combine_like_terms(right);
            (
                lhs_coefficient * rhs_coefficient,
                Node::BinaryOp {
                    left: Box::new(lhs_product),
                    operation: OperationType::Multiply,
                    right: Box::new(rhs_product),
                },
            )
        }
        Node::Number(n) => (*n, expression.clone()),
        _ => (1.0, expression.clone()),
    };
    println!("Test: {} -> {:?}", coefficient, product);
    //    if let Node::Variable(var) = product {
    //        (coefficient, Node::BinaryOp { left: Box::new(Node::Number(coefficient.into())), operation: OperationType::Multiply, right: Box::new(Node::Variable(var)) } )
    //    } else {
    //        (coefficient, product)
    //    }
    if coefficient == 1.0 {
        (coefficient, product)
    } else {
        (
            coefficient,
            Node::BinaryOp {
                left: Box::new(Node::Number(coefficient)),
                operation: OperationType::Multiply,
                right: Box::new(product),
            },
        )
    }
}
