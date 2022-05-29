use std::io::Write;

use crate::{ast_printer::AstPrinter, expr::Visitor, lexer::Lexer, parser::Parser};

static mut HAD_ERROR: bool = false;

pub fn run_file(file_path: &str) {
    let file_data = match std::fs::read_to_string(file_path) {
        Ok(data) => data,
        Err(e) => {
            println!("{}", e);
            std::process::exit(64);
        }
    };

    run(&file_data);
}

pub fn run_interactive() {
    loop {
        unsafe { HAD_ERROR = false };
        print!(":> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Error reading line");

        if input.is_empty() {
            break;
        }

        run(input.trim());
    }
}

pub fn run(source: &str) {
    let lexer = Lexer::new(source);
    let tokens = lexer.collect_tokens();

    if unsafe { HAD_ERROR } {
        return;
    }

    let mut parser = Parser::new(tokens);
    let expression = parser.parse();

    if unsafe { HAD_ERROR } {
        return;
    }

    println!("{}", (AstPrinter {}).visit_expr(&expression.unwrap()));
}

pub fn report_error() {
    unsafe { HAD_ERROR = true };
}
