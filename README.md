# rslox - Implementation of the lox language proposed by the book “Crafting Interpreters”.
This repository is an example of one of the ways in which the `lox` programming language could be implemented by following the steps described in the book [***"crafting interpreters"***](https://craftinginterpreters.com/) by *Robert Nystrom*

## How to Use

### Interactive Mode
To run the interpreter in interactive mode, use the following command:
```bash
cargo run -q
```
This will start the interpreter where you can type and execute code directly.

### Run From a File
To execute code from a file, use:
```bash
cargo run -q -- <file_path>
```
Replace `<file_path>` with the path to your `.lox` file.

## Learning log

### Dynamic types [2025-01-10]
Rust as an innovate language compared to many others by providing a safe and efficient way to manage memory through concepts like **ownership**, **borrowing**, **lifetimes**, and the combined management of the **stack** and the **heap** 

For example, in Rust, it's possible to handle dynamic data flexible and securely using structures like the following:
```rust
pub struct Token { 
    literal: Box<dyn any::Any>,
}
```

In this case:
- **`Box`** allows the data to be stored on the **heap**, ensured that the Rust automatically handles its allocation and deallocation
- **`dyn`** indicates that the specific type of the value is not know at the compile time.
- **`any::Any`** allows the `literal` field to store values of any time that implements the `Any` trait.

### Output buffering [2025-01-06]
In many programming languages, data is temporarily stored in a location called a `buffer` before begin send to the console. This buffer holds the data until it's ready to be displayed

But this data is only displayed when one of these conditions is met
- The `buffer` is full
- A newline `\n` is entered
- **(Rust)** `flush` is explicitly called