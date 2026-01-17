use std::{
    env, fs,
    io::{stdin, stdout, Write},
};

use vm::exec::VM;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        eprintln!("Usage: rslox <path> ");
    }
}

fn repl() {
    let mut line = String::new();

    loop {
        print!("> ");
        if let Err(err) = stdout().flush() {
            eprint!("I/O error: {err:?}");
        };

        if let Err(err) = stdin().read_line(&mut line) {
            eprint!("Readline error: {err:?}");
        }

        if line.trim().is_empty() {
            print!("\n");
            break;
        }

        VM::interpret(&line);

        line.clear();
    }
}

fn run_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Err(e) => return eprintln!("File error: {e}"),
        Ok(s) => s,
    };

    VM::interpret(&source);
}
