use std::{env, path::PathBuf};

use clap::{builder::styling, Command, Parser};
use cli::{
    alerts::{show_help, Alert},
    commands::{Cli, Commands, ToolCommand},
};
use lox::{run_file, run_prompt};
use owo_colors::{style, OwoColorize};
use tools::AstGenerator;

mod cli;
mod errors;
mod lox;
mod tools;
mod utils;

fn main() {

    let command = Command::new("hello");
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run {
            path,
            debug,
            show_ast,
            show_tokens,
        } => {}
        Commands::Tool { command } => {
            handle_tool_command(command);
        }
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

fn handle_tool_command(tool_type: &ToolCommand) {
    match tool_type {
        ToolCommand::GenAst { output_path } => handle_gen_ast_tool(output_path),
    }
}

// endregion: Command handlers

// region: Subcommand handlers

fn handle_gen_ast_tool(output_path: &String) {
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

    AstGenerator::new(base_name, ast_types).gen(output_path);
    Alert::success(String::from("CLI | AST successfully created")).show();
}

// endregion: Subcommand handlers
