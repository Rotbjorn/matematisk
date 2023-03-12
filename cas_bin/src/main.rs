use std::{io::Write, fs::File};

use cas_lib::cas::{lexer::Lexer, parser::Parser, token::Token, node::{Node, OperationType}, simplify::combine_like_terms};


fn main() {
    //let mut test_node = Node::BinaryOp { left: Box::new(Node::BinaryOp { left: Box::new(Node::Number(2.0)), operation: OperationType::Multiply, right: Box::new(Node::Variable("x".to_string())) }), operation: OperationType::Multiply, right: Box::new(Node::Number(3.0)) };
    //let (co, p) = combine_like_terms(&mut test_node);
    repl();
    //println!("{:?}\n{}, -- {:?}", test_node, co, p)
}

fn repl() {
    // TODO: Use better library for readline to allow for readline editing (arrows, CTRL+arrow etc)
    use std::io::{stdin, stdout};
    let stdin = stdin();
    let mut stdout = stdout();
    loop {
        let mut buf = String::new();
        print!("\n> ");
        stdout.flush().expect("Couldn't flush!");

        stdin.read_line(&mut buf).expect("Error reading line!");
        buf = buf.trim().to_string();
        // TODO: Add proper command handling
        if buf.starts_with(":lex") {
            let expression = &buf[5..];

            for tok in Lexer::new(expression) {
                println!("{:?}", tok);
            }
        } else if buf.starts_with(":parse") {
            let expression = &buf[7..];
            let tokens: Vec<Token> = Lexer::new(expression).collect();
            let result = Parser::new(tokens).parse();
            let Ok(ast) = result else {
                let error = result.err().unwrap();
                println!("{:?}", error);
                continue;
            };
            println!(" -- Abstract Syntax Tree --\n{:?}",ast);
            let mut graph_buf = String::new();
            ast.render_dot_graph_notation(&mut graph_buf);
            println!("Graph generated --- \n{}", graph_buf);
            let mut file = File::create("AST.dot").expect("Couldn't create file");
            file.write(graph_buf.as_bytes()).expect("Couldn't write to dot file!");
        } else if buf.starts_with(":q") {
            break;
        }
    }
    return;
}