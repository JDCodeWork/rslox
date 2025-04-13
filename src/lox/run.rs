use std::{fs, io, process, ptr::null};

use crate::{
    cli::Alert,
    errors::{Error, SystemError},
    lox::scanner::Scanner,
    tools::AstPrinter,
};

use super::{
    expr::{self, Binary, Expr, Grouping, Literal, Unary},
    parser::Parser,
    token::Token,
};

pub fn run_file(path: String) {
    let path_formatted = if path.ends_with(".lox") {
        Alert::warning("CLI | It's not necessary to include .lox extension".to_string()).show();
        path
    } else if path.contains('.') {
        Error::from(SystemError::InvalidFileExtension).report_and_exit(1);
    } else {
        format!("{path}.lox")
    };

    let raw_code = match fs::read_to_string(&path_formatted) {
        Ok(val) => val,
        Err(..) => Error::from(SystemError::FileNotFound(path_formatted)).report_and_exit(1),
    };

    if let Err(e) = run(&raw_code) {
        e.report_and_exit(67);
    }
}

pub fn run_prompt() {
    let mut line = String::new();

    loop {
        line.clear();
        print!("> ");

        // Force the buffer to be send to the console
        if let Err(e) = io::Write::flush(&mut io::stdout()) {
            Error::from(SystemError::Io(e)).report();
        }

        if let Err(e) = io::stdin().read_line(&mut line) {
            Error::from(SystemError::Io(e)).report();
        }

        if line.is_empty() {
            print!("\n");
            break;
        }

        run(&line).map_err(|e| e.report()).ok();
    }
}

fn run(raw_code: &str) -> Result<(), Error> {
    let mut scanner = Scanner::new(raw_code.to_string());
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens.clone());

    /* for token in tokens {
        Alert::info(token.to_string()).show();
    } */

    let expr = match parser.parse() {
        Ok(expr) => expr,
        Err(lox_error) => Error::from(lox_error).report_and_exit(1),
    };

    print!("\n");
    Alert::info(format!("AST -> {}", AstPrinter::print(expr))).show();

    Ok(())
}
