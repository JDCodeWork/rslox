use std::{collections::BTreeMap, fs, io};

use crate::{
    cli::alerts::Alert,
    errors::{Error, SystemError},
    lox::{scanner::Scanner, token::{Token, TokenType}},
    tools::AstPrinter,
};

use super::parser::Parser;

pub struct RunOptsCommand {
    pub debug: bool,
    pub show_ast: bool,
    pub show_tokens: bool,
}
impl Default for RunOptsCommand {
    fn default() -> Self {
        RunOptsCommand {
            debug: false,
            show_ast: false,
            show_tokens: false,
        }
    }
}

pub fn handle_run_command(path: Option<String>, opts: RunOptsCommand) {
    let RunOptsCommand {
        debug,
        show_ast,
        show_tokens,
    } = opts;

    let source: String;

    if let Some(path) = path {
        let valid_path = handle_path_format(&path);
        source = read_file(&valid_path);
    } else {
        Alert::info("CLI | No file path provided, reading from prompt...".to_string()).show();
        Alert::info("CLI | To exit, press Enter on an empty line.".to_string()).show();

        source = read_prompt();
        
        if source.trim().is_empty() {
            return;
        }
    }

    let mut scanner = Scanner::new(source.to_string());
    let tokens = scanner.scan_tokens();

    if debug && !show_ast && !show_tokens {
        Alert::info("CLI | Debug mode is enabled.".to_string()).show();
        debug_show_tokens(tokens.clone());
        debug_show_ast(tokens.clone());
    }

    if show_ast {
        debug_show_ast(tokens.clone());
    }

    if show_tokens {
        debug_show_tokens(tokens.clone());
    }

    run(tokens.clone()).unwrap();
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
        if line.trim().is_empty() {
            print!("\n");
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

fn debug_show_tokens(tokens: Vec<Token>) {
    for token in tokens {
        Alert::info(token.to_string()).show();
    }
}

fn debug_show_ast(tokens: Vec<Token>) {
    let mut tokens_by_line: BTreeMap<usize, Vec<Token>> = BTreeMap::new();

    for token in tokens {
        tokens_by_line
            .entry(token.get_line() as usize)
            .or_default()
            .push(token);
    }

    for (line, line_tokens) in tokens_by_line {
        // Skip empty lines or lines with only EOF token
        if line_tokens.is_empty() || 
           (line_tokens.len() == 1 && *line_tokens[0].get_type() == TokenType::EOF) {
            continue;
        }
        
        let mut parser = Parser::new(line_tokens.clone());
        match parser.parse() {
            Ok(expr) => {
                Alert::info(format!("AST (line {line}) -> {}", AstPrinter::print(expr))).show();
            }
            Err(lox_error) => {
                Error::from(lox_error).report_and_exit(1);
            }
        }
    }
}
