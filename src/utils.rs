use css::DarkGray;
use owo_colors::{colors::*, OwoColorize};

pub fn show_help() {
    println!("\n{}", " USAGE ".fg::<Black>().bg::<Green>());
    println!(
        "\n{} {} {} {}",
        "$".fg::<DarkGray>(),
        "rslox".fg::<Green>(),
        "<COMMAND>".fg::<Cyan>(),
        "[OPTION]".fg::<Yellow>()
    );

    println!("\n{}\n", " COMMANDS ".fg::<Black>().bg::<Blue>());
    show_command("run", "run lox code");
    show_command("gen", "use some tool to debug");

    println!("\n{}\n", " OPTIONS ".fg::<Black>().bg::<Yellow>());
    print!("{} {} ", "$".fg::<DarkGray>(), "rslox".fg::<Green>());
    println!("{}\t{}", "--help".yellow(), "Show help info");

    print!("{} {} ", "$".fg::<DarkGray>(), "rslox".fg::<Green>());
    println!(
        "{} {} {}\t{}",
        "run".fg::<Blue>().italic(),
        "-p".yellow(),
        "<PATH>".fg::<DarkGray>().italic(),
        "Path of the file to run"
    )
}

fn show_command(name: &str, desc: &str) {
    println!("{}\t{}", name.blue(), desc.italic());
}
