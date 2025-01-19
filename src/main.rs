use std::env;

use cli::{show_help, Alert};
use lox::{run_file, run_prompt};

mod cli;
mod errors;
mod lox;
mod utils;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    if args.len() < 1 {
        return show_help();
    }

    match args[0].as_str() {
        "run" => {
            if args.get(1).is_none() {
                run_prompt();
            } else if args[1] == "-p" && args.get(2).is_some() {
                run_file(args[2].to_string());
            } else {
                Alert::error(format!("CLI | Not valid option: '{}'", args[1])).show_and_exit(1)
            }
        }
        _ => Alert::error(format!("CLI | Not valid command: '{}'", args[0])).show_and_exit(1),
    }
}
