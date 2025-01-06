use std::{env, fs};

fn run(raw_code: String) {
    println!("Your code is\n\n{}", raw_code)
}

fn run_file(path: String){
    let raw_code = fs::read_to_string(path).expect("Failed to read the file");
    run(raw_code);
}

fn main() {
    let mut args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Run Interactive CLI")
    } else if args.len() == 2 {
        run_file(args.remove(1));
    } else {
        println!("Usage: rslox <file_path>")
    }
}
