use std::env;

use lox::{run_file, run_prompt};
use utils::show_help;

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
                show_help();
            }
        }
        _ => {
            show_help();
        }
    }
}
