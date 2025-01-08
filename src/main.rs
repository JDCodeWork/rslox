use std::{env, fs, io};

use utils::error;

mod utils;

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
    let raw_code = fs::read_to_string(path).expect("Failed to read the file");
    run(&raw_code);
}

fn run_prompt() {
    let mut line = String::new();

    loop {
        line.clear();
        print!("> ");

        // Force the buffer to be send to the console
        io::Write::flush(&mut io::stdout()).expect("Failed to clean buffer");

        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");

        if line.is_empty() {
            break;
        }

        run(&line);
    }
}

fn run(raw_code: &str) {
    println!("| {}", raw_code);
}
