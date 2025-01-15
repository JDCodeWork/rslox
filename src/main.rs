#![allow(warnings)]
use std::{env, fs, io, process};

use errors::{Error, SystemError};
use scanner::Scanner;

mod errors;
mod scanner;
mod token;

fn main() {
    let mut args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        run_prompt();
    } else if args.len() == 2 {
        run_file(args.remove(1));
    } else {
        println!("Usage: rslox <file_path>")
    }
}

fn run_file(path: String) {
    let path_formatted = if path.ends_with(".lox") {
        path
    } else if path.contains('.') {
        Error::from(SystemError::InvalidFileExtension).report();
        process::exit(1);
    } else {
        format!("{path}.lox")
    };

    let raw_code = match fs::read_to_string(&path_formatted) {
        Ok(val) => val,
        Err(..) => {
            Error::from(SystemError::FileNotFound(path_formatted)).report();
            process::exit(1)
        }
    };

    if let Err(e) = run(&raw_code) {
        e.report();
        process::exit(67)
    }
}

fn run_prompt() {
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

    for token in tokens {
        println!("-> {}", token)
    }

    println!();
    Ok(())
}
