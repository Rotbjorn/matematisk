#[derive(Debug, Clone)]
pub enum Node {
    Program(Vec<Node>),
    Number(f64),
    Variable(String),

    List(Vec<Node>),
    Function {
        name: String,
        function_body: Box<Node>,
    },

    // TODO: Break into addition, multiply, power
    BinaryOp {
        left: Box<Node>,
        operation: OperationType,
        right: Box<Node>,
    },
}

#[derive(Debug, Clone)]
pub enum OperationType {
    Add,
    Multiply,
    Power,
}

impl Node {
    pub fn render_dot_graph_notation(&self, out: &mut String) {
        out.push_str("digraph AST {\nlabel = \"Abstract Syntax Tree\"\n");
        let mut count: u32 = 0;
        self.render_dot_graph_notation_impl(out, &mut count);
        out.push('}');
    }

    fn render_dot_graph_notation_impl(&self, out: &mut String, count: &mut u32) -> u32 {
        let current_node_id = *count;
        match self {
            Node::Program(v) | Node::List(v) => {
                out.push_str(format!("N_{} [label = \"<>\"]\n", current_node_id).as_str());
                *count += 1;
                for (index, node) in v.iter().enumerate() {
                    out.push_str(format!("S_{0} [label = \"{0}\"]\n", index).as_str());
                    out.push_str(format!("N_{} -> S_{}\n", current_node_id, index).as_str());
                    let target_id = node.render_dot_graph_notation_impl(out, count);
                    out.push_str(format!("S_{} -> N_{}\n", index, target_id).as_str());
                }
            }
            Node::Number(n) => {
                out.push_str(format!("N_{} [label = \"num: {}\"]\n", current_node_id, n).as_str());
                *count += 1;
            }
            Node::Variable(s) => {
                out.push_str(format!("N_{} [label = \"var: {}\"]\n", current_node_id, s).as_str());
                *count += 1;
            }
            Node::Function {
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
            Node::BinaryOp {
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

                out.push_str(
                    format!("N_{} -> {{ N_{} N_{} }}\n", current_node_id, lhs_id, rhs_id).as_str(),
                );
            }
        }
        current_node_id
    }
}
