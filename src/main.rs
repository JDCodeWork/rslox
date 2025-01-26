use std::env;

use cli::{show_help, Alert};
use lox::{run_file, run_prompt};
use tools::AstGenerator;

mod cli;
mod errors;
mod lox;
mod tools;
mod utils;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    if args.len() < 1 {
        return show_help();
    }

    match args[0].as_str() {
        "run" => handle_run_command(args),
        "tool" => handle_tool_command(args),
        _ => Alert::error(format!("CLI | Not valid command: '{}'", args[0])).show_and_exit(1),
    }
}

// region: Command handlers

fn handle_run_command(args: Vec<String>) {
    if args.get(1).is_none() {
        run_prompt();
    } else if args[1] == "-p" && args.get(2).is_some() {
        run_file(args[2].to_string());
    } else {
        Alert::error(format!("CLI | Not valid option: '{}'", args[1])).show_and_exit(1)
    }
}

fn handle_tool_command(args: Vec<String>) {
    let tool = match args.get(1) {
        Some(t) => t.as_str(),
        None => Alert::error(String::from("CLI | You must specify the name of the tool "))
            .show_and_exit(1),
    };

    match tool {
        "gen-ast" => handle_gen_ast_tool(args),
        _ => Alert::error(format!("CLI | Tool '{}' not found", args[1])).show_and_exit(1),
    }
}

// endregion: Command handlers

// region: Subcommand handlers

fn handle_gen_ast_tool(args: Vec<String>) {
    let path = match args.get(2) {
        Some(p) => p.to_string(),
        None => {
            Alert::error(String::from("CLI | You must enter the output directory")).show_and_exit(1)
        }
    };

    let base_name = String::from("Expr");
    let ast_types = vec![
        "Binary   : Expr left, Token operator, Expr right",
        "Grouping : Expr expression",
        "Literal  : String value",
        "Unary    : Token operator, Expr right",
    ]
    .iter()
    .map(|t| t.to_string())
    .collect();

    AstGenerator::new(base_name, ast_types).gen(path);
    Alert::success(String::from("CLI | AST successfully created")).show();
}

// endregion: Subcommand handlers
