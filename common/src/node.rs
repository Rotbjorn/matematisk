
#[derive(Debug, Clone)]
pub enum Statement {
    Program(Vec<Statement>),
    Function { name: String, function_body: Expr },
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    Variable(String),

    List(Vec<Expr>),

    // TODO: Break into addition, multiply, power
    BinaryOp {
        left: Box<Expr>,
        operation: OperationType,
        right: Box<Expr>,
    },
}

impl Statement {
    pub fn render_dot_graph_notation(&self, out: &mut String) {
        out.push_str("digraph AST {\n");
        out.push_str("label = \"Abstract Syntax Tree\"\n");
        out.push_str("fontname = \"Arial\"\n");
        out.push_str("node [fontname = \"Arial\"]\n");
        out.push_str("edge [fontname = \"Arial\"]\n");
        let mut count: u32 = 0;
        self.render_dot_graph_notation_impl(out, &mut count);
        out.push('}');
    }

    fn render_dot_graph_notation_impl(&self, out: &mut String, count: &mut u32) -> u32 {
        let current_node_id = *count;
        use Statement::*;
        match self {
            Program(v) => {
                out.push_str(format!("N_{} [label = \"<>\"]\n", current_node_id).as_str());
                *count += 1;
                for (index, node) in v.iter().enumerate() {
                    out.push_str(format!("S_{0} [label = \"{0}\"]\n", index).as_str());
                    out.push_str(format!("N_{} -> S_{}\n", current_node_id, index).as_str());
                    let target_id = node.render_dot_graph_notation_impl(out, count);
                    out.push_str(format!("S_{} -> N_{}\n", index, target_id).as_str());
                }
            }
            Function {
                name,
                function_body,
            } => {
                out.push_str(
                    format!("N_{} [label = \"func: {}\"]\n", current_node_id, name).as_str(),
                );
                *count += 1;

                let body_id = function_body.render_dot_graph_notation_impl(out, count);

                out.push_str(format!("N_{} ->  N_{} \n", current_node_id, body_id).as_str());
            }
            Expression(ex) => {
                let _target_id = ex.render_dot_graph_notation_impl(out, count);
                *count += 1;
            }
        }
        current_node_id
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    Add,
    Multiply,
    Power,
}

impl Expr {
    pub fn render_dot_graph_notation_impl(&self, out: &mut String, count: &mut u32) -> u32 {
        let current_node_id = *count;
        use Expr::*;
        match self {
            List(v) => {
                out.push_str(format!("N_{} [label = \"<>\"]\n", current_node_id).as_str());
                *count += 1;
                for (index, node) in v.iter().enumerate() {
                    out.push_str(format!("S_{0} [label = \"{0}\"]\n", index).as_str());
                    out.push_str(format!("N_{} -> S_{}\n", current_node_id, index).as_str());
                    let target_id = node.render_dot_graph_notation_impl(out, count);
                    out.push_str(format!("S_{} -> N_{}\n", index, target_id).as_str());
                }
            }
            Number(n) => {
                out.push_str(format!("N_{} [label = \"num: {}\"]\n", current_node_id, n).as_str());
                *count += 1;
            }
            Variable(s) => {
                out.push_str(format!("N_{} [label = \"var: {}\"]\n", current_node_id, s).as_str());
                *count += 1;
            }
            BinaryOp {
                left,
                operation,
                right,
            } => {
                out.push_str(
                    format!("N_{} [label = \"{:?}\"]\n", current_node_id, operation).as_str(),
                );
                *count += 1;

                let lhs_id = left.render_dot_graph_notation_impl(out, count);
                let rhs_id = right.render_dot_graph_notation_impl(out, count);

                if *operation == OperationType::Power {
                    out.push_str(
                        format!("N_{0} -> N_{1} [label = \"base\"]\nN_{0} -> N_{2} [label = \"exp\"]\n", current_node_id, lhs_id, rhs_id).as_str(),
                    );
                } else {

                    out.push_str(
                        format!("N_{} -> {{ N_{} N_{} }}\n", current_node_id, lhs_id, rhs_id).as_str(),
                    );
                }
            }
        }
        current_node_id
    }
}
