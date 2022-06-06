pub mod ast_printer;
pub mod common;
pub mod environment;
pub mod expr;
pub mod interpreter;
pub mod lexer;
pub mod lox;
pub mod parser;
pub mod stmt;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author="ObiWanWheeler", version="0.0.1", about="An interpreter for the Lox language specification, found at https://github.com/munificent/craftinginterpreters", long_about = None)]
struct Args {
    #[clap(short, long)]
    file_path: Option<String>,
}

fn main() {
    let args = Args::parse();

    match args.file_path {
        Some(fp) => {
            lox::run_file(&fp);
        }
        None => {
            lox::run_interactive();
        }
    }
}
