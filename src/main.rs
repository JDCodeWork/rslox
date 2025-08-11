use clap::Parser;
use cli::{
    alerts::Alert,
    commands::{Cli, Commands, ToolCommand},
};
use lox::{run_file, run_prompt};
use tools::AstGenerator;

mod cli;
mod errors;
mod lox;
mod tools;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run {
            path: Some(path),
            debug: _,
            show_ast: _,
            show_tokens: _,
        } => handle_run_command(path),
        Commands::Run {
            path: None,
            debug: _,
            show_ast: _,
            show_tokens: _,
        } => run_prompt(),
        Commands::Tool { command } => {
            handle_tool_command(command);
        }
    }
}

// region: Command handlers

fn handle_run_command(path: &str) {
    run_file(path.to_string());
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
