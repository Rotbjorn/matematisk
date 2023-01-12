use std::{io::Write, fs::File};

use cas_lib::cas::{lexer::Lexer, parser::Parser, token::Token};


fn main() {
    repl();
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
            let ast = Parser::new(tokens).parse();
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