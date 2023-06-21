use std::{
    fs::{self, File},
    io::{ErrorKind, Write},
    process::exit,
};

use matex_common::node::ASTGraphGenerator;
use matex_compiler::cas::{
    eval::runtime::Runtime,
    syntax::{lexer, parser},
};

use clap::{Parser, Subcommand};
use repl::Repl;

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
    env_logger::builder().format_timestamp(None).init();
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

        let Ok(program) = result else {
            let error = result.err().unwrap();
            eprintln!("{}", error);
            exit(-1);
        };

        println!("Program: --------\n{:?}\n--------", program);

        if let Some(outfile_path) = args.ast {
            let mut graph_buf = String::new();

            ASTGraphGenerator::new(&mut graph_buf)
                .create_dot_graph(&program)
                .expect("Failed to create graph");

            println!("Graph generated --- \n{}", graph_buf);

            let mut file = File::create(outfile_path).expect("Couldn't create file");
            file.write_all(graph_buf.as_bytes())
                .expect("Couldn't write to dot file!");
        }

        let mut runtime = Runtime::new();

        let exit_value = runtime.run(&program);

        println!("EXIT VALUE: {:?}", exit_value);
    } else {
        let mut repl = Repl::new();
        let _ = repl.run();
    }
}
