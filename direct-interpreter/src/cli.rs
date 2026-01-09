pub mod commands {
    use clap::{Parser, Subcommand};

    #[derive(Parser)]
    #[command(name = "rslox", about = "Lox interpreter written in Rust")]
    pub struct Cli {
        #[command(subcommand)]
        pub command: Commands,
    }

    #[derive(Subcommand)]
    pub enum Commands {
        /// Executes a file or starts the REPL
        Run {
            /// Path to the Lox file to execute
            #[arg(short, long, value_name = "FILE_PATH")]
            path: Option<String>,

            /// Controlled debug mode
            #[arg(short, long)]
            debug: bool,

            /// Display the generated AST
            #[arg(long)]
            show_ast: bool,

            /// Display the generated tokens
            #[arg(long)]
            show_tokens: bool,
        },

        /// Development helper tools
        Tool {
            #[command(subcommand)]
            command: ToolCommand,
        },
    }

    #[derive(Subcommand)]
    pub enum ToolCommand {
        /// Generates the AST from definitions and saves it to the given output path
        GenAst {
            /// Output path for the generated AST file
            #[arg(value_name = "output_path")]
            output_path: String,
        },
    }
}

#[allow(dead_code)]
pub mod alerts {
    use std::process;

    use owo_colors::{AnsiColors, DynColors, OwoColorize};

    pub struct Alert {
        name: String,
        msg: String,
    }

    enum AlertType {
        Success,
        Warning,
        Info,
        Error,
    }

    impl Alert {
        pub fn success(msg: String) -> Self {
            let a_type = AlertType::Success;

            Alert::new_generic(a_type, msg)
        }

        pub fn info(msg: String) -> Self {
            let a_type = AlertType::Info;

            Alert::new_generic(a_type, msg)
        }

        pub fn warning(msg: String) -> Self {
            let a_type = AlertType::Warning;

            Alert::new_generic(a_type, msg)
        }

        pub fn error(msg: String) -> Self {
            let a_type = AlertType::Error;

            Alert::new_generic(a_type, msg)
        }

        fn get_name(from_type: &AlertType) -> String {
            match from_type {
                AlertType::Error => String::from(" ERROR "),
                AlertType::Warning => String::from(" WARNING "),
                AlertType::Info => String::from(" INFO "),
                AlertType::Success => String::from(" SUCCESS "),
            }
        }

        fn get_color(from_type: &AlertType) -> DynColors {
            match from_type {
                AlertType::Error => DynColors::Ansi(AnsiColors::Red),
                AlertType::Warning => DynColors::Ansi(AnsiColors::Yellow),
                AlertType::Info => DynColors::Ansi(AnsiColors::Blue),
                AlertType::Success => DynColors::Ansi(AnsiColors::Green),
            }
        }

        fn new_generic(from_type: AlertType, msg: String) -> Self {
            let color = Alert::get_color(&from_type);

            let name = Alert::get_name(&from_type).on_color(color).to_string();
            let msg = msg.color(color).to_string();

            Self { name, msg }
        }
    }

    impl Alert {
        pub fn show(&self) {
            let Self { name, msg, .. } = self;

            println!("{name} {msg}")
        }

        pub fn show_and_exit(&self, code: i32) -> ! {
            let Self { name, msg, .. } = self;

            println!("{name} {msg}");
            process::exit(code)
        }
    }
}
