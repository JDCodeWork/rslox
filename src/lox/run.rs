use std::{collections::BTreeMap, fs, io};

use crate::{
    cli::alerts::Alert,
    errors::{Err, IoErr},
    lox::{
        interpreter::Interpreter,
        resolver::Resolver,
        scanner::Scanner,
        token::{Token, TokenType},
    },
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

    let tokens = Scanner::scan_from(source.to_string());

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

    if let Err(lang_err) = run(tokens.clone()) {
        lang_err.report_and_exit(1)
    }
}

fn handle_path_format(path: &str) -> String {
    if path.ends_with(".lox") {
        path.to_string()
    } else {
        IoErr::InvalidFileExtension.to_err().report_and_exit(1);
    }
}

fn read_file(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(val) => val,
        Err(..) => IoErr::FileNotFound(path.to_string())
            .to_err()
            .report_and_exit(1),
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
            IoErr::Sys(e).to_err().report();
        }

        if let Err(e) = io::stdin().read_line(&mut line) {
            IoErr::Sys(e).to_err().report();
        }
        if line.trim().is_empty() {
            print!("\n");
            break;
        }

        source.push_str(&line);
    }

    source
}

fn run(tokens: Vec<Token>) -> Result<(), Err> {
    let mut parser = Parser::new(tokens);

    let statements = match parser.parse() {
        Ok(expr) => expr,
        Err(lox_err) => {
            // Report parse error and attempt to recover so REPL can continue
            lox_err.report();
            return Ok(());
        }
    };
    let mut resolver = Resolver::new(Interpreter::new());
    resolver.resolve_stmts(statements.clone());

    let mut interpreter = resolver.interpreter;
    match interpreter.interpret(statements) {
        Ok(()) => (),
        Err(runtime_err) => return Err(runtime_err),
    };

    Ok(())
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
        if line_tokens.is_empty()
            || (line_tokens.len() == 1 && *line_tokens[0].get_type() == TokenType::EOF)
        {
            continue;
        }

        let mut parser = Parser::new(line_tokens.clone());
        match parser.parse() {
            Ok(stmts) => {
                for stmt in stmts {
                    Alert::info(format!("AST (line {line}) -> {}", AstPrinter::print(stmt))).show();
                }
            }
            Err(lox_error) => {
                // Report and continue to next line instead of exiting
                lox_error.report();
                continue;
            }
        }
    }
}
