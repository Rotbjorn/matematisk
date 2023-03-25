use std::{fs::File, io::Write, str::FromStr};

use matex_common::node::{Statement, ASTGraphGenerator, Visitor};
use matex_compiler::cas::{
    backend::runtime::RuntimeVisitor,
    frontend::{lexer, parser},
};
use rustyline::error::ReadlineError;

pub fn repl() -> Result<(), ReadlineError> {
    let mut rl = rustyline::DefaultEditor::new()?;

    loop {
        let input = rl.readline("matex > ")?.trim().to_string();

        if input.starts_with(":") {
            let Ok(command) = input.parse::<Command>() else {
                eprintln!("Unknown command!");
                continue;
            };

            let result = command.execute();

            match result {
                CommandResult::Exit => break,
                CommandResult::None => {}
            }
            continue;
        }

        match parser::Parser::new(lexer::Lexer::new(&input).collect()).parse() {
            Ok(ast) => {
                let exit_value = RuntimeVisitor::default().visit_statement(&ast);

                println!("{:?}", exit_value);
            }
            Err(e) => {
                eprintln!("Error occurred:\n{}", e);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Command {
    input: String,
    cmd_type: CommandType,
}

impl Command {
    pub fn new(command_type: CommandType, input: &String) -> Self {
        Self {
            cmd_type: command_type,
            input: input.clone(),
        }
    }

    pub fn execute(&self) -> CommandResult {
        match self.cmd_type {
            CommandType::Lexer => {
                for tok in lexer::Lexer::new(&self.input) {
                    println!("{}", tok);
                }
            }

            CommandType::Parser => {
                let Ok(ast) = self.run_parser() else {
                    return CommandResult::None
                };

                println!(" -- Abstract Syntax Tree --\n{:?}", ast);
            }

            CommandType::GenerateDot => {
                let Ok(ast) = self.run_parser() else {
                    return CommandResult::None
                };

                let mut graph_buf = String::new();

                ASTGraphGenerator::new(&mut graph_buf).create_dot_graph(&ast).expect("Failed to create graph");

                println!("Graph generated --- \n{}", graph_buf);

                let mut file = File::create("AST.dot").expect("Couldn't create file");
                file.write_all(graph_buf.as_bytes())
                    .expect("Couldn't write to dot file!");
            }
            CommandType::Exit => return CommandResult::Exit,
        }
        CommandResult::None
    }

    fn run_parser(&self) -> Result<Statement, ()> {
        let tokens = lexer::Lexer::new(&self.input).collect();
        let result = parser::Parser::new(tokens).parse();

        let Ok(ast) = result else {
                    let error = result.err().unwrap();
                    eprintln!("{:?}", error);
                    return Err(());
                };

        return Ok(ast);
    }
}

pub enum CommandResult {
    Exit,
    None,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.strip_prefix(":").ok_or(())?;

        let (command, args) = input.split_once(' ').ok_or(())?;

        let command = command.parse::<CommandType>()?;

        Ok(Command::new(command, &args.to_string()))
    }
}

#[derive(Debug, Clone)]
pub enum CommandType {
    Lexer,
    Parser,
    GenerateDot,
    Exit,
}

impl FromStr for CommandType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lexer" | "l" => Ok(CommandType::Lexer),
            "parser" | "p" => Ok(CommandType::Parser),
            "dot" => Ok(CommandType::GenerateDot),
            "quit" | "q" => Ok(CommandType::Exit),
            _ => Err(()),
        }
    }
}