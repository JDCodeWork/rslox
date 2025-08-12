use std::{fs, io};

use crate::{
    cli::alerts::Alert,
    errors::{Error, SystemError},
    lox::{scanner::Scanner, token::Token},
    tools::AstPrinter,
};

use super::parser::Parser;

pub struct RunOptsCommand {
    debug: Option<bool>,
    show_ast: Option<bool>,
    show_tokes: Option<bool>,
}
impl Default for RunOptsCommand {
    fn default() -> Self {
        RunOptsCommand {
            debug: None,
            show_ast: None,
            show_tokes: None,
        }
    }
}

pub fn handle_run_command(path: Option<String>, opts: Option<RunOptsCommand>) {
    let mut source: String;

    if let Some(path) = path {
        let valid_path = handle_path_format(&path);
        source = read_file(&valid_path);
    } else {
        // TODO: When interactive mode is enable, notify the user to the exit, they should type 'exit' and press Enter
        source = read_prompt();
    }

    let mut scanner = Scanner::new(source.to_string());
    let tokens = scanner.scan_tokens();

    let RunOptsCommand {
        debug,
        show_ast,
        show_tokes,
    } = opts.unwrap_or_default();

    // TODO: If debug mode is enable and none of the other settings have a value, execute all settings as if they were true
    if let Some(true) = debug {
        for token in tokens {
            println!("{token}");
        }
    }

    run(tokens.clone());
}

fn handle_path_format(path: &str) -> String {
    if path.ends_with(".lox") {
        Alert::warning("CLI | It's not necessary to include .lox extension".to_string()).show();
        path.to_string()
    } else if path.to_string().contains('.') {
        Error::from(SystemError::InvalidFileExtension).report_and_exit(1);
    } else {
        format!("{path}.lox")
    }
}

fn read_file(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(val) => val,
        Err(..) => Error::from(SystemError::FileNotFound(path.to_string())).report_and_exit(1),
    }
}

fn read_prompt() -> String {
    let mut source = String::new();
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

        if line.trim() == "exit" {
            break;
        }

        source.push_str(&line);
    }

    source
}

fn run(tokens: Vec<Token>) -> Result<(), Error> {
    let mut parser = Parser::new(tokens);

    match parser.parse() {
        Ok(..) => Ok(()),
        Err(lox_error) => Error::from(lox_error).report_and_exit(1),
    }
}
