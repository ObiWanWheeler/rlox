mod lexer;
use lexer::Lexer;

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
            run_file(&fp);
        }
        None => {
            run_interactive();
        }
    }
}

fn run_file(file_path: &str) {
    let file_data = match std::fs::read_to_string(file_path) {
        Ok(data) => data,
        Err(e) => {
            println!("{}", e);
            std::process::exit(64);
        }
    };

    if let Err(e) = run(&file_data) {
        panic!("{:?}", e);
    }
}

fn run_interactive() {
    loop {
        print!(":> ");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Error reading line");

        if input.is_empty() {
            break;
        }

        if let Err(e) = run(input.trim()) {
            println!("{:?}", e);
        }
    }
}

fn run(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let lexer = Lexer::new(source);
    let tokens = lexer.collect_tokens()?;

    println!("{:?}", tokens);

    Ok(())
}
