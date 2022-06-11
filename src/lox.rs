use crate::{interpreter::Interpreter, lexer::Lexer, parser::Parser, resolver::Resolver};
use std::{io::Write, cell::RefCell, rc::Rc};

static mut HAD_ERROR: bool = false;
static mut HAD_RUNTIME_ERROR: bool = false;

pub fn run_file(file_path: &str) {
    let file_data = match std::fs::read_to_string(file_path) {
        Ok(data) => data,
        Err(e) => {
            println!("{}", e);
            std::process::exit(64);
        }
    };

    run(&file_data, Rc::new(RefCell::new(Interpreter::new())));
}

pub fn run_interactive() {
    let interpreter = Rc::new(RefCell::new(Interpreter::new()));
    loop {
        unsafe { HAD_ERROR = false };
        unsafe { HAD_RUNTIME_ERROR = false };
        print!(":> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Error reading line");

        if input.is_empty() {
            break;
        }

        run(input.trim(), Rc::clone(&interpreter));
    }
}

pub fn run(source: &str, interpreter: Rc<RefCell<Interpreter>>) {
    let lexer = Lexer::new(source);
    let tokens = lexer.collect_tokens();

    if unsafe { HAD_ERROR } {
        return;
    }

    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    if unsafe { HAD_ERROR } {
        return;
    }
    
    let mut resolver = Resolver::new(Rc::clone(&interpreter));
    resolver.resolve(&statements);

    if unsafe { HAD_ERROR } {
        return;
    }

    interpreter.borrow_mut().interpret(&statements);
}

pub fn report_error() {
    unsafe { HAD_ERROR = true };
}

pub fn report_runtime_error() {
    unsafe { HAD_RUNTIME_ERROR = true };
}
