use std::{
    fs::{self, File},
    io::{ErrorKind, Write},
    process::exit,
};

use matex_common::node::{ASTGraphGenerator, Visitor};
use matex_compiler::cas::{
    backend::runtime::Runtime,
    frontend::{lexer, parser},
};

use clap::{Parser, Subcommand};
use repl::REPL;

mod repl;

#[derive(Parser)]
#[command(author = "Rotbjorn", version = "0.1.0")]
struct Args {
    path: Option<std::path::PathBuf>,
    #[arg(short, long)]
    ast: Option<std::path::PathBuf>,
    //#[command(subcommand)]
    //command: Option<Commands>
}

#[derive(Subcommand)]
enum Commands {
    File { path: Option<std::path::PathBuf> },
}

fn main() {
    let args = Args::parse();
    if let Some(path) = args.path {
        let read_file = fs::read_to_string(&path);

        let contents = read_file.unwrap_or_else(|e| {
            let error = match e.kind() {
                ErrorKind::NotFound => format!("File {:?} not found.", path),
                ErrorKind::PermissionDenied => {
                    format!("Couldn't read '{:?}': No permissions.", path)
                }
                _ => "Other error".to_string(),
            };
            eprintln!("{}", error);
            exit(-1);
        });

        let tokens = lexer::Lexer::new(&contents).collect();
        let mut parser = parser::Parser::new(tokens);

        let result = parser.parse();

        dbg!(&parser.parsed);

        let Ok(ast) = result else {
            let error = result.err().unwrap();
            panic!("{:?}", error);
        };

        println!("Node: --------\n{:?}\n--------", ast);

        if let Some(outfile_path) = args.ast {
            let mut graph_buf = String::new();

            ASTGraphGenerator::new(&mut graph_buf)
                .create_dot_graph(&ast)
                .expect("Failed to create graph");

            println!("Graph generated --- \n{}", graph_buf);

            let mut file = File::create(outfile_path).expect("Couldn't create file");
            file.write_all(graph_buf.as_bytes())
                .expect("Couldn't write to dot file!");
        }

        let mut runtime = Runtime::default();

        let exit_value = runtime.visit_statement(&ast);

        println!("EXIT VALUE: {:?}", exit_value);
    } else {
        let mut repl = REPL::default();
        let _ = repl.run();
    }
}
