#[derive(Debug)]
pub enum Node {
    Number(f64),
    Variable(String),

    List(Vec<Node>),

    // TODO: Break into addition, multiply, power
    BinaryOp {
        left: Box<Node>,
        operation: OperationType,
        right: Box<Node>,
    },
}

#[derive(Debug)]
pub enum OperationType {
    Add,
    Multiply,
    Power,
}



impl Node {
    pub fn render_dot_graph_notation(&self, out: &mut String) {
        out.push_str(
            "digraph AST {\nlabel = \"Abstract Syntax Tree\"\n",
        );
        let mut count: u32 = 0; 
        self.render_dot_graph_notation_impl(out, &mut count);
        out.push_str("}");
    }

    fn render_dot_graph_notation_impl(&self, out: &mut String, count: &mut u32) -> u32 {
        let current_node_id = *count;
        match self {
            Node::Number(n) => {
                out.push_str(format!("Node_{} [label = \"{}\"]\n", current_node_id, n).as_str());
                *count += 1;
            },
            Node::Variable(s) => {
                out.push_str(format!("Node_{} [label = \"{}\"", current_node_id, s).as_str());
                *count += 1;
            }, 
            Node::List(v) => {
                *count += 1;
                for node in v {
                    let target_id = node.render_dot_graph_notation_impl(out, count);
                    out.push_str(format!("Node_{} -> Node_{}\n", current_node_id, target_id).as_str())
                }
            },
            Node::BinaryOp { left, operation, right } => {
                out.push_str(format!("Node_{} [label = \"{:?}\"]\n", current_node_id, operation).as_str());
                *count += 1;

                let lhs_id = left.render_dot_graph_notation_impl(out, count);
                let rhs_id = right.render_dot_graph_notation_impl(out, count);

                out.push_str(format!("Node_{} -> {{ Node_{} Node_{} }}\n", current_node_id, lhs_id, rhs_id).as_str());
            },
        }
        return current_node_id
    }
}
